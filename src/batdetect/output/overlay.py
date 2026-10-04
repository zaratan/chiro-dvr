from __future__ import annotations

import itertools

import cv2
import numpy as np

from batdetect.detect import Detection
from batdetect.output.config import RenderConfig
from batdetect.track import Track
from batdetect.video import ColorFrame, VideoInfo

BOX_COLOR = (0, 0, 255)


TRAIL_COLOR = (0, 200, 255)


def _box(det: Detection, pad: int) -> tuple[tuple[int, int], tuple[int, int]]:
    x0 = int(det.left) - pad
    y0 = int(det.top) - pad
    x1 = int(det.left + det.width) + pad
    y1 = int(det.top + det.height) + pad
    return (x0, y0), (x1, y1)


def interpolate(a: Detection, b: Detection, frame: int) -> Detection:
    t = (frame - a.frame) / (b.frame - a.frame)

    def lerp(u: float, v: float) -> float:
        return u + (v - u) * t

    return Detection(
        frame,
        lerp(a.x, b.x),
        lerp(a.y, b.y),
        lerp(a.left, b.left),
        lerp(a.top, b.top),
        lerp(a.width, b.width),
        lerp(a.height, b.height),
        lerp(a.area, b.area),
        0.0,
    )


def filled_points(track: Track) -> list[tuple[Detection, bool]]:
    filled: list[tuple[Detection, bool]] = [(track.first, False)]
    for a, b in itertools.pairwise(track.points):
        filled.extend((interpolate(a, b, f), True) for f in range(a.frame + 1, b.frame))
        filled.append((b, False))
    return filled


def draw_overlay(
    frame: ColorFrame, hits: list[tuple[Track, Detection, bool]], info: VideoInfo, cfg: RenderConfig, frame_no: int
) -> None:
    trail_frames = round(cfg.trail_s * info.fps)
    for track, det, interpolated in hits:
        top_left, bottom_right = _box(det, cfg.box_pad)
        cv2.rectangle(frame, top_left, bottom_right, BOX_COLOR, 1 if interpolated else 2)
        label_at = (top_left[0], top_left[1] - 6)
        cv2.putText(frame, f"#{track.id}", label_at, cv2.FONT_HERSHEY_SIMPLEX, 0.8, BOX_COLOR, 2)
        trail = [p for p in track.points if frame_no - trail_frames <= p.frame <= frame_no]
        if len(trail) > 1:
            pts = np.array([(round(p.x), round(p.y)) for p in trail], dtype=np.int32)
            cv2.polylines(frame, [pts], isClosed=False, color=TRAIL_COLOR, thickness=2)
