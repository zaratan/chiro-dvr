from __future__ import annotations

import argparse

from batdetect.detect import DetectConfig, Region
from batdetect.stability import StabilityConfig
from batdetect.track import TrackConfig

MODES = {
    "normal": DetectConfig(),
    "quick": DetectConfig(work_width=480, threshold=25, target_sigma=0, noise_factor=0),
}
DEFAULT_MODE = "normal"


def parse_region(text: str) -> Region:
    try:
        x0, y0, x1, y1 = (float(v) for v in text.split(","))
        return Region(x0, y0, x1, y1)
    except ValueError as err:
        raise argparse.ArgumentTypeError(f"expected x0,y0,x1,y1 fractions, got {text!r}: {err}") from err


def mode_help(name: str) -> str:
    m = MODES[name]
    return (
        f"{name}: {m.work_width} px, threshold {m.threshold:g}, "
        f"target sigma {m.target_sigma:g}, noise factor {m.noise_factor:g}"
    )


def chosen[T](explicit: T | None, mode_value: T) -> T:
    return mode_value if explicit is None else explicit


def positive_int(text: str) -> int:
    value = int(text)
    if value < 1:
        raise argparse.ArgumentTypeError(f"must be >= 1, got {value}")
    return value


def add_detection_arguments(ap: argparse.ArgumentParser) -> None:
    d = DetectConfig()
    detect = ap.add_argument_group("detection")
    detect.add_argument(
        "--mode",
        choices=tuple(MODES),
        default=DEFAULT_MODE,
        help=f"{mode_help('normal')} (default); {mode_help('quick')}; explicit options win",
    )
    detect.add_argument("--threshold", type=float, help="default set by --mode")
    detect.add_argument("--min-area", type=float, default=d.min_area, help="source pixels²")
    detect.add_argument("--max-area", type=float, default=d.max_area, help="source pixels²")
    detect.add_argument("--bg-window", type=float, default=d.bg_window_s, help="seconds of rolling median background")
    detect.add_argument("--bg-step", type=int, default=d.bg_step)
    detect.add_argument(
        "--osd-region",
        type=parse_region,
        action="append",
        default=[],
        help="x0,y0,x1,y1 as fractions of the frame; masks a moving on-screen display",
    )
    detect.add_argument(
        "--merge-radius", type=float, default=d.merge_radius, help="source pixels bridged between fragments"
    )
    detect.add_argument("--work-width", type=int, help="default set by --mode")
    detect.add_argument(
        "--noise-factor",
        type=float,
        help="raises --threshold to this many times the local noise of each pixel; 0 disables",
    )
    detect.add_argument(
        "--target-sigma",
        type=float,
        help="source pixels; Gaussian blur matched to the target size before the background; 0 disables",
    )
    s = StabilityConfig()
    detect.add_argument(
        "--max-blobs",
        type=int,
        default=s.max_blobs,
        help="frames with this many blobs above the usual level are ignored, as when the camera moves; 0 disables",
    )
    detect.add_argument("--unstable-pad", type=float, default=s.pad_s, help="seconds ignored around such frames")


def add_tracking_arguments(ap: argparse.ArgumentParser) -> None:
    t = TrackConfig()
    track = ap.add_argument_group("tracking")
    track.add_argument("--max-jump", type=float, default=t.max_jump, help="source pixels")
    track.add_argument("--max-gap", type=int, default=t.max_gap)
    track.add_argument("--min-hits", type=int, default=t.min_hits)
    track.add_argument("--min-travel", type=float, default=t.min_travel, help="source pixels")
    track.add_argument(
        "--twin-distance", type=float, default=t.twin_distance, help="source pixels between fragments of one animal"
    )
    track.add_argument(
        "--max-median-turn",
        type=float,
        default=t.max_median_turn,
        help="radians; tracks that zigzag more are noise; 3.15 or more disables",
    )


def build_detect_config(ns: argparse.Namespace) -> DetectConfig:
    mode = MODES[ns.mode]
    return DetectConfig(
        threshold=chosen(ns.threshold, mode.threshold),
        min_area=ns.min_area,
        max_area=ns.max_area,
        bg_window_s=ns.bg_window,
        bg_step=ns.bg_step,
        osd_regions=tuple(ns.osd_region),
        merge_radius=ns.merge_radius,
        work_width=chosen(ns.work_width, mode.work_width),
        noise_factor=chosen(ns.noise_factor, mode.noise_factor),
        target_sigma=chosen(ns.target_sigma, mode.target_sigma),
    )


def build_stability_config(ns: argparse.Namespace) -> StabilityConfig:
    return StabilityConfig(max_blobs=ns.max_blobs, pad_s=ns.unstable_pad)


def build_track_config(ns: argparse.Namespace) -> TrackConfig:
    return TrackConfig(
        max_jump=ns.max_jump,
        max_gap=ns.max_gap,
        min_hits=ns.min_hits,
        min_travel=ns.min_travel,
        twin_distance=ns.twin_distance,
        max_median_turn=ns.max_median_turn,
    )
