from __future__ import annotations

import argparse

from batdetect.detect import DetectConfig, Region
from batdetect.track import TrackConfig


def parse_region(text: str) -> Region:
    try:
        x0, y0, x1, y1 = (float(v) for v in text.split(","))
        return Region(x0, y0, x1, y1)
    except ValueError as err:
        raise argparse.ArgumentTypeError(f"expected x0,y0,x1,y1 fractions, got {text!r}: {err}") from err


def add_detection_arguments(ap: argparse.ArgumentParser) -> None:
    d = DetectConfig()
    detect = ap.add_argument_group("detection")
    detect.add_argument("--threshold", type=float, default=d.threshold)
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
    detect.add_argument("--work-width", type=int, default=d.work_width)


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


def build_detect_config(ns: argparse.Namespace) -> DetectConfig:
    return DetectConfig(
        threshold=ns.threshold,
        min_area=ns.min_area,
        max_area=ns.max_area,
        bg_window_s=ns.bg_window,
        bg_step=ns.bg_step,
        osd_regions=tuple(ns.osd_region),
        merge_radius=ns.merge_radius,
        work_width=ns.work_width,
    )


def build_track_config(ns: argparse.Namespace) -> TrackConfig:
    return TrackConfig(
        max_jump=ns.max_jump,
        max_gap=ns.max_gap,
        min_hits=ns.min_hits,
        min_travel=ns.min_travel,
        twin_distance=ns.twin_distance,
    )
