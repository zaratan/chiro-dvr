from __future__ import annotations

import itertools

import cv2
import numpy as np

from batdetect.detect import Detection
from batdetect.output.clips import ClipWindow, View
from batdetect.output.config import RenderConfig
from batdetect.output.names import zoom_path
from batdetect.output.overlay import TRAIL_COLOR, filled_points
from batdetect.output.zoom import SLOW_MOTION, Crop, needs_zoom, zoom_crop
from batdetect.track import Track
from batdetect.video import ColorFrame, VideoInfo

TRAIL_GAP_PX = 16
TRAIL_THICKNESS = 1

type Point = tuple[float, float]
type Segment = tuple[Point, Point]


def _inside_span(a: Point, b: Point, low: Point, high: Point) -> tuple[float, float] | None:
    t0, t1 = 0.0, 1.0
    for axis in (0, 1):
        delta = b[axis] - a[axis]
        if delta == 0:
            if not low[axis] <= a[axis] <= high[axis]:
                return None
            continue
        u, v = (low[axis] - a[axis]) / delta, (high[axis] - a[axis]) / delta
        t0, t1 = max(t0, min(u, v)), min(t1, max(u, v))
    return (t0, t1) if t0 < t1 else None


def _at(a: Point, b: Point, t: float) -> Point:
    return a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t


def _outside(a: Point, b: Point, low: Point, high: Point) -> list[Segment]:
    span = _inside_span(a, b, low, high)
    if span is None:
        return [(a, b)]
    t0, t1 = span
    return [part for part, kept in (((a, _at(a, b, t0)), t0 > 0), ((_at(a, b, t1), b), t1 < 1)) if kept]


def trail_segments(track: Track, current: Detection, trail_frames: int) -> list[Segment]:
    start = current.frame - trail_frames
    path = [(p.x, p.y) for p in track.points if start <= p.frame < current.frame] + [(current.x, current.y)]
    low = (current.left - TRAIL_GAP_PX, current.top - TRAIL_GAP_PX)
    high = (current.left + current.width + TRAIL_GAP_PX, current.top + current.height + TRAIL_GAP_PX)
    return [part for a, b in itertools.pairwise(path) if a != b for part in _outside(a, b, low, high)]


def _on_screen(point: Point, crop: Crop, scale: float) -> tuple[int, int]:
    return round((point[0] - crop.left + 0.5) * scale - 0.5), round((point[1] - crop.top + 0.5) * scale - 0.5)


def zoom_view(track: Track, crop: Crop, info: VideoInfo, cfg: RenderConfig) -> View:
    positions = {det.frame: det for det, _ in filled_points(track)}
    trail_frames = round(cfg.trail_s * info.fps)
    scale = info.width / crop.width

    def view(frame: ColorFrame, frame_no: int) -> ColorFrame:
        region = frame[crop.top : crop.top + crop.height, crop.left : crop.left + crop.width]
        zoomed = np.asarray(
            cv2.resize(region, (info.width, info.height), interpolation=cv2.INTER_NEAREST), dtype=np.uint8
        )
        current = positions.get(frame_no)
        if current is not None:
            for a, b in trail_segments(track, current, trail_frames):
                start, end = _on_screen(a, crop, scale), _on_screen(b, crop, scale)
                cv2.line(zoomed, start, end, TRAIL_COLOR, TRAIL_THICKNESS, cv2.LINE_AA)
        return zoomed

    return view


def zoom_windows(tracks: list[Track], clips: list[ClipWindow], info: VideoInfo, cfg: RenderConfig) -> list[ClipWindow]:
    return [
        ClipWindow(
            clip.first,
            clip.last,
            zoom_path(clip.path),
            zoom_view(track, zoom_crop(track, info.width, info.height), info, cfg),
            SLOW_MOTION,
        )
        for track, clip in zip(tracks, clips, strict=True)
        if needs_zoom(track, cfg)
    ]
