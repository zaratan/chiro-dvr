from __future__ import annotations

import itertools
import math
from collections import deque
from collections.abc import Iterable
from dataclasses import dataclass

import cv2
import numpy as np
import numpy.typing as npt

from batdetect.median import temporal_median
from batdetect.video import GrayFrame

MIN_BACKGROUND_FRAMES = 3
MIN_WORK_WIDTH = 16


@dataclass(frozen=True, slots=True)
class Region:
    x0: float
    y0: float
    x1: float
    y1: float

    def __post_init__(self) -> None:
        if not (0 <= self.x0 < self.x1 <= 1 and 0 <= self.y0 < self.y1 <= 1):
            raise ValueError("a region must satisfy 0 <= x0 < x1 <= 1 and 0 <= y0 < y1 <= 1")

    def contains(self, x: float, y: float) -> bool:
        return self.x0 <= x <= self.x1 and self.y0 <= y <= self.y1


@dataclass(frozen=True, slots=True)
class DetectConfig:
    threshold: float = 25
    min_area: float = 4
    max_area: float = 2700
    bg_window_s: float = 1.0
    bg_step: int = 3
    osd_regions: tuple[Region, ...] = ()
    merge_radius: float = 6
    work_width: int = 480

    def __post_init__(self) -> None:
        if self.threshold <= 0:
            raise ValueError("threshold must be > 0")
        if not 0 < self.min_area <= self.max_area:
            raise ValueError("expected 0 < min_area <= max_area")
        if self.bg_window_s <= 0:
            raise ValueError("bg_window_s must be > 0")
        if self.merge_radius < 0:
            raise ValueError("merge_radius must be >= 0")
        if self.bg_step < 1:
            raise ValueError("bg_step must be >= 1")
        if self.work_width < MIN_WORK_WIDTH:
            raise ValueError(f"work_width must be >= {MIN_WORK_WIDTH}")

    def half_window(self, fps: float) -> int:
        half = round(self.bg_window_s * fps / 2)
        if 2 * half + 1 < MIN_BACKGROUND_FRAMES:
            raise ValueError(f"bg_window_s={self.bg_window_s} covers fewer than {MIN_BACKGROUND_FRAMES} frames")
        return half


@dataclass(frozen=True, slots=True)
class Detection:
    frame: int
    x: float
    y: float
    left: float
    top: float
    width: float
    height: float
    area: float
    amplitude: float


def osd_mask(width: int, height: int, cfg: DetectConfig) -> npt.NDArray[np.bool_]:
    mask = np.ones((height, width), dtype=np.bool_)
    for region in cfg.osd_regions:
        x0, x1 = math.floor(region.x0 * width), math.ceil(region.x1 * width)
        y0, y1 = math.floor(region.y0 * height), math.ceil(region.y1 * height)
        mask[y0:y1, x0:x1] = False
    return mask


def find_blobs(residual: npt.NDArray[np.float64], frame: int, cfg: DetectConfig, scale: float) -> list[Detection]:
    above = np.abs(residual) > cfg.threshold
    merged = above.astype(np.uint8)
    merge_radius = round(cfg.merge_radius / scale)
    if merge_radius > 0:
        size = 2 * merge_radius + 1
        kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (size, size))
        merged = np.asarray(cv2.morphologyEx(merged, cv2.MORPH_CLOSE, kernel), dtype=np.uint8)
    count, labels_mat, stats_mat, centroids_mat = cv2.connectedComponentsWithStats(merged)
    labels = np.asarray(labels_mat)
    stats = np.asarray(stats_mat, dtype=np.int64)
    centroids = np.asarray(centroids_mat, dtype=np.float64)
    found: list[Detection] = []
    for i in range(1, count):
        pixels = above & (labels == i)
        area = float(pixels.sum()) * scale**2
        if not cfg.min_area <= area <= cfg.max_area:
            continue
        left, top, width, height = (float(v) * scale for v in stats[i, :4])
        amplitude = float(np.abs(residual[pixels]).max())
        cx, cy = ((float(v) + 0.5) * scale - 0.5 for v in centroids[i])
        found.append(Detection(frame, cx, cy, left, top, width, height, area, amplitude))
    return found


def detect_frames(
    frames: Iterable[GrayFrame], fps: float, cfg: DetectConfig, scale: float = 1.0
) -> dict[int, list[Detection]]:
    half = cfg.half_window(fps)
    iterator = iter(frames)
    first = next(iterator, None)
    if first is None:
        return {}
    mask = osd_mask(first.shape[1], first.shape[0], cfg)
    if not mask.any():
        raise ValueError("osd_regions mask the whole frame")
    window: deque[tuple[int, npt.NDArray[np.int16], float]] = deque(maxlen=2 * half + 1)
    detections: dict[int, list[Detection]] = {}

    def process(target: int) -> None:
        entries = [e for e in window if abs(e[0] - target) <= half]
        current = next(e for e in entries if e[0] == target)
        sampled = entries[:: cfg.bg_step]
        background = temporal_median([e[1] for e in sampled])
        offset = current[2] - float(np.mean([e[2] for e in sampled]))
        residual = current[1] - background - offset
        residual[~mask] = 0
        detections[target] = find_blobs(residual, target, cfg, scale)

    index = -1
    for index, gray in enumerate(itertools.chain([first], iterator)):
        window.append((index, gray.astype(np.int16), float(gray[mask].mean())))
        if index - half >= 0:
            process(index - half)
    for target in range(max(0, index - half + 1), index + 1):
        process(target)
    return detections
