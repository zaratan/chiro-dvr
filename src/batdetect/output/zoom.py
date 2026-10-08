from __future__ import annotations

import math
from dataclasses import dataclass

from batdetect.output.config import ZOOM_ALL, ZOOM_NONE, RenderConfig
from batdetect.track import Track

ZOOM_MARGIN_PX = 40
MAX_MAGNIFICATION = 4
SLOW_MOTION = 4


@dataclass(frozen=True, slots=True)
class Crop:
    left: int
    top: int
    width: int
    height: int


def needs_zoom(track: Track, cfg: RenderConfig) -> bool:
    if cfg.zoom == ZOOM_ALL:
        return True
    if cfg.zoom == ZOOM_NONE:
        return False
    return track.max_area() < cfg.zoom_below_area or track.peak_amplitude() < cfg.zoom_below_amplitude


def _placed(center: float, size: int, limit: int) -> int:
    return min(max(0, round(center - size / 2)), limit - size)


def zoom_crop(track: Track, width: int, height: int) -> Crop:
    x0 = min(p.left for p in track.points) - ZOOM_MARGIN_PX
    y0 = min(p.top for p in track.points) - ZOOM_MARGIN_PX
    x1 = max(p.left + p.width for p in track.points) + ZOOM_MARGIN_PX
    y1 = max(p.top + p.height for p in track.points) + ZOOM_MARGIN_PX
    units = math.gcd(width, height)
    unit_w, unit_h = width // units, height // units
    needed = max(math.ceil((x1 - x0) / unit_w), math.ceil((y1 - y0) / unit_h), math.ceil(units / MAX_MAGNIFICATION))
    count = min(needed, units)
    crop_w, crop_h = count * unit_w, count * unit_h
    return Crop(_placed((x0 + x1) / 2, crop_w, width), _placed((y0 + y1) / 2, crop_h, height), crop_w, crop_h)
