from __future__ import annotations

import argparse
from collections.abc import Callable

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


def per_mode(value: Callable[[DetectConfig], float]) -> str:
    return ", ".join(f"{name}: {value(config):g}" for name, config in MODES.items())


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
        help="normal sees smaller and fainter targets; quick needs about half the computing time but can miss them; "
        "an option given explicitly wins over the mode (default: %(default)s)",
    )
    detect.add_argument(
        "--threshold",
        type=float,
        metavar="LEVEL",
        help="smallest difference from the background that counts as a target; lower finds fainter targets "
        f"and more noise; grey levels ({per_mode(lambda m: m.threshold)})",
    )
    detect.add_argument(
        "--min-area",
        type=float,
        default=d.min_area,
        metavar="PIXELS2",
        help="smaller spots are dropped as noise; source pixels² (default: %(default)s)",
    )
    detect.add_argument(
        "--max-area",
        type=float,
        default=d.max_area,
        metavar="PIXELS2",
        help="larger spots are dropped; source pixels² (default: %(default)s)",
    )
    detect.add_argument(
        "--bg-window",
        type=float,
        default=d.bg_window_s,
        metavar="SECONDS",
        help="the background is the median of this much video around each frame; a target that stays still "
        "for more than half of it blends into it; seconds (default: %(default)s)",
    )
    detect.add_argument(
        "--bg-step",
        type=int,
        default=d.bg_step,
        metavar="FRAMES",
        help="the background uses one frame in this many; higher is faster but leaves fewer frames "
        "to measure the noise of each pixel; frames (default: %(default)s)",
    )
    detect.add_argument(
        "--osd-region",
        type=parse_region,
        action="append",
        default=[],
        metavar="X0,Y0,X1,Y1",
        help="hides a moving on-screen display from the detection, as fractions of the frame; "
        "repeat for several (default: none)",
    )
    detect.add_argument(
        "--merge-radius",
        type=float,
        default=d.merge_radius,
        metavar="PIXELS",
        help="spots up to about twice this distance apart become one detection, so a fragmented target counts once; "
        "source pixels (default: %(default)s)",
    )
    detect.add_argument(
        "--work-width",
        type=int,
        metavar="PIXELS",
        help="width the frames are reduced to for detection; wider sees smaller targets but takes longer; "
        f"pixels ({per_mode(lambda m: m.work_width)})",
    )
    detect.add_argument(
        "--noise-factor",
        type=float,
        metavar="FACTOR",
        help="raises --threshold to this many times the noise of each pixel, so noisy areas give fewer "
        f"false tracks; 0 disables ({per_mode(lambda m: m.noise_factor)})",
    )
    detect.add_argument(
        "--target-sigma",
        type=float,
        metavar="PIXELS",
        help="blur matched to the size of a small target before the comparison with the background, so faint "
        f"targets stand out from the noise; source pixels; 0 disables ({per_mode(lambda m: m.target_sigma)})",
    )
    s = StabilityConfig()
    detect.add_argument(
        "--max-blobs",
        type=int,
        default=s.max_blobs,
        metavar="COUNT",
        help="frames with at least this many spots more than six times the usual count are ignored, "
        "as when the binoculars move; "
        "0 disables (default: %(default)s)",
    )
    detect.add_argument(
        "--unstable-pad",
        type=float,
        default=s.pad_s,
        metavar="SECONDS",
        help="time also ignored on each side of such frames; seconds (default: %(default)s)",
    )


def add_tracking_arguments(ap: argparse.ArgumentParser) -> None:
    t = TrackConfig()
    track = ap.add_argument_group("tracking")
    track.add_argument(
        "--max-jump",
        type=float,
        default=t.max_jump,
        metavar="PIXELS",
        help="farthest a detection may lie from where a track was expected and still extend it; "
        "source pixels (default: %(default)s)",
    )
    track.add_argument(
        "--max-gap",
        type=int,
        default=t.max_gap,
        metavar="FRAMES",
        help="frames in a row without a detection that a track survives, so a faint target that briefly "
        "disappears keeps one track; frames (default: %(default)s)",
    )
    track.add_argument(
        "--min-hits",
        type=int,
        default=t.min_hits,
        metavar="COUNT",
        help="tracks with fewer detections are dropped; lower keeps shorter tracks and more short flickers "
        "(default: %(default)s)",
    )
    track.add_argument(
        "--min-travel",
        type=float,
        default=t.min_travel,
        metavar="PIXELS",
        help="tracks that move less from start to end are dropped, like a pixel flickering in place; "
        "source pixels (default: %(default)s)",
    )
    track.add_argument(
        "--twin-distance",
        type=float,
        default=t.twin_distance,
        metavar="PIXELS",
        help="two tracks that stay this close on all their shared frames are merged as one animal; "
        "source pixels; 0 disables (default: %(default)s)",
    )
    track.add_argument(
        "--max-median-turn",
        type=float,
        default=t.max_median_turn,
        metavar="RADIANS",
        help="tracks that zigzag more are dropped as noise; radians; 3.15 or more disables (default: %(default)s)",
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
