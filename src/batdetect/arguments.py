from __future__ import annotations

import argparse
from collections.abc import Callable
from pathlib import Path

from batdetect.detect import DetectConfig, Region
from batdetect.language import _, decimal
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
        message = _("expected x0,y0,x1,y1 fractions, got {text}: {error}").format(text=repr(text), error=err)
        raise argparse.ArgumentTypeError(message) from err


def per_mode(value: Callable[[DetectConfig], float]) -> str:
    return _(", ").join(
        _("{mode}: {value}").format(mode=name, value=decimal(value(config))) for name, config in MODES.items()
    )


def with_modes(text: str, value: Callable[[DetectConfig], float]) -> str:
    return f"{text} ({per_mode(value)})"


def with_default(text: str, default: float | str | Path) -> str:
    shown = decimal(default) if isinstance(default, float) else str(default)
    return _("{help} (default: {value})").format(help=text, value=shown)


def chosen[T](explicit: T | None, mode_value: T) -> T:
    return mode_value if explicit is None else explicit


def positive_int(text: str) -> int:
    value = int(text)
    if value < 1:
        raise argparse.ArgumentTypeError(_("must be >= 1, got {value}").format(value=value))
    return value


def add_detection_arguments(ap: argparse.ArgumentParser) -> None:
    d = DetectConfig()
    detect = ap.add_argument_group(_("detection"))
    detect.add_argument(
        "--mode",
        choices=tuple(MODES),
        default=DEFAULT_MODE,
        help=with_default(
            _(
                "normal sees smaller and fainter targets; quick needs about half the computing time "
                "but can miss them; an option given explicitly wins over the mode"
            ),
            DEFAULT_MODE,
        ),
    )
    detect.add_argument(
        "--threshold",
        type=float,
        metavar=_("LEVEL"),
        help=with_modes(
            _(
                "smallest difference from the background that counts as a target; lower finds fainter targets "
                "and more noise; grey levels"
            ),
            lambda m: m.threshold,
        ),
    )
    detect.add_argument(
        "--min-area",
        type=float,
        default=d.min_area,
        metavar=_("PIXELS2"),
        help=with_default(_("smaller spots are dropped as noise; source pixels²"), d.min_area),
    )
    detect.add_argument(
        "--max-area",
        type=float,
        default=d.max_area,
        metavar=_("PIXELS2"),
        help=with_default(_("larger spots are dropped; source pixels²"), d.max_area),
    )
    detect.add_argument(
        "--bg-window",
        type=float,
        default=d.bg_window_s,
        metavar=_("SECONDS"),
        help=with_default(
            _(
                "the background is the median of this much video around each frame; a target that stays still "
                "for more than half of it blends into it; seconds"
            ),
            d.bg_window_s,
        ),
    )
    detect.add_argument(
        "--bg-step",
        type=int,
        default=d.bg_step,
        metavar=_("FRAMES"),
        help=with_default(
            _(
                "the background uses one frame in this many; higher is faster but leaves fewer frames "
                "to measure the noise of each pixel; frames"
            ),
            d.bg_step,
        ),
    )
    detect.add_argument(
        "--osd-region",
        type=parse_region,
        action="append",
        default=[],
        metavar="X0,Y0,X1,Y1",
        help=with_default(
            _("hides a moving on-screen display from the detection, as fractions of the frame; repeat for several"),
            _("none"),
        ),
    )
    detect.add_argument(
        "--merge-radius",
        type=float,
        default=d.merge_radius,
        metavar=_("PIXELS"),
        help=with_default(
            _(
                "spots up to about twice this distance apart become one detection, so a fragmented target "
                "counts once; source pixels"
            ),
            d.merge_radius,
        ),
    )
    detect.add_argument(
        "--work-width",
        type=int,
        metavar=_("PIXELS"),
        help=with_modes(
            _("width the frames are reduced to for detection; wider sees smaller targets but takes longer; pixels"),
            lambda m: m.work_width,
        ),
    )
    detect.add_argument(
        "--noise-factor",
        type=float,
        metavar=_("FACTOR"),
        help=with_modes(
            _(
                "raises --threshold to this many times the noise of each pixel, so noisy areas give fewer "
                "false tracks; 0 disables"
            ),
            lambda m: m.noise_factor,
        ),
    )
    detect.add_argument(
        "--target-sigma",
        type=float,
        metavar=_("PIXELS"),
        help=with_modes(
            _(
                "blur matched to the size of a small target before the comparison with the background, so faint "
                "targets stand out from the noise; source pixels; 0 disables"
            ),
            lambda m: m.target_sigma,
        ),
    )
    s = StabilityConfig()
    detect.add_argument(
        "--max-blobs",
        type=int,
        default=s.max_blobs,
        metavar=_("COUNT"),
        help=with_default(
            _(
                "frames with at least this many spots more than six times the usual count are ignored, "
                "as when the binoculars move; 0 disables"
            ),
            s.max_blobs,
        ),
    )
    detect.add_argument(
        "--unstable-pad",
        type=float,
        default=s.pad_s,
        metavar=_("SECONDS"),
        help=with_default(_("time also ignored on each side of such frames; seconds"), s.pad_s),
    )


def add_tracking_arguments(ap: argparse.ArgumentParser) -> None:
    t = TrackConfig()
    track = ap.add_argument_group(_("tracking"))
    track.add_argument(
        "--max-jump",
        type=float,
        default=t.max_jump,
        metavar=_("PIXELS"),
        help=with_default(
            _("farthest a detection may lie from where a track was expected and still extend it; source pixels"),
            t.max_jump,
        ),
    )
    track.add_argument(
        "--max-gap",
        type=int,
        default=t.max_gap,
        metavar=_("FRAMES"),
        help=with_default(
            _(
                "frames in a row without a detection that a track survives, so a faint target that briefly "
                "disappears keeps one track; frames"
            ),
            t.max_gap,
        ),
    )
    track.add_argument(
        "--min-hits",
        type=int,
        default=t.min_hits,
        metavar=_("COUNT"),
        help=with_default(
            _("tracks with fewer detections are dropped; lower keeps shorter tracks and more short flickers"),
            t.min_hits,
        ),
    )
    track.add_argument(
        "--min-travel",
        type=float,
        default=t.min_travel,
        metavar=_("PIXELS"),
        help=with_default(
            _("tracks that move less from start to end are dropped, like a pixel flickering in place; source pixels"),
            t.min_travel,
        ),
    )
    track.add_argument(
        "--twin-distance",
        type=float,
        default=t.twin_distance,
        metavar=_("PIXELS"),
        help=with_default(
            _(
                "two tracks that stay this close on all their shared frames are merged as one animal; "
                "source pixels; 0 disables"
            ),
            t.twin_distance,
        ),
    )
    track.add_argument(
        "--max-median-turn",
        type=float,
        default=t.max_median_turn,
        metavar=_("RADIANS"),
        help=with_default(
            _("tracks that zigzag more are dropped as noise; radians; 3.15 or more disables"), t.max_median_turn
        ),
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
