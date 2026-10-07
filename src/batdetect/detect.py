from __future__ import annotations

import itertools
import math
from collections import deque
from collections.abc import Iterable, Sequence
from dataclasses import dataclass

import cv2
import numpy as np
import numpy.typing as npt

from batdetect.median import NumericFrame, temporal_median
from batdetect.noise import temporal_noise
from batdetect.video import GrayFrame

MIN_BACKGROUND_FRAMES = 3
MIN_WORK_WIDTH = 16
MIN_NOISE_SAMPLES = 7
BLUR_REACH = 3

PixelThreshold = float | npt.NDArray[np.float64]


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
    threshold: float = 12
    min_area: float = 4
    max_area: float = 2700
    bg_window_s: float = 1.0
    bg_step: int = 3
    osd_regions: tuple[Region, ...] = ()
    merge_radius: float = 6
    work_width: int = 960
    noise_factor: float = 8
    target_sigma: float = 1.5

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
        if not (math.isfinite(self.noise_factor) and self.noise_factor >= 0):
            raise ValueError("noise_factor must be a finite number >= 0")
        if not (math.isfinite(self.target_sigma) and self.target_sigma >= 0):
            raise ValueError("target_sigma must be a finite number >= 0")

    def half_window(self, fps: float) -> int:
        half = round(self.bg_window_s * fps / 2)
        window = 2 * half + 1
        if window < MIN_BACKGROUND_FRAMES:
            raise ValueError(f"bg_window_s={self.bg_window_s} covers fewer than {MIN_BACKGROUND_FRAMES} frames")
        if len(range(0, window, self.bg_step)) < MIN_BACKGROUND_FRAMES:
            raise ValueError(
                f"bg_step={self.bg_step} keeps fewer than {MIN_BACKGROUND_FRAMES} of {window} window frames"
            )
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


def osd_mask(width: int, height: int, cfg: DetectConfig, margin: int = 0) -> npt.NDArray[np.bool_]:
    mask = np.ones((height, width), dtype=np.bool_)
    for region in cfg.osd_regions:
        x0, x1 = max(0, math.floor(region.x0 * width) - margin), math.ceil(region.x1 * width) + margin
        y0, y1 = max(0, math.floor(region.y0 * height) - margin), math.ceil(region.y1 * height) + margin
        mask[y0:y1, x0:x1] = False
    return mask


def find_blobs(
    residual: npt.NDArray[np.float64],
    frame: int,
    cfg: DetectConfig,
    scale: float,
    threshold: PixelThreshold | None = None,
) -> list[Detection]:
    magnitude = np.abs(residual)
    above = magnitude > (cfg.threshold if threshold is None else threshold)
    merged = above.astype(np.uint8)
    merge_radius = round(cfg.merge_radius / scale)
    if merge_radius > 0:
        size = 2 * merge_radius + 1
        kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (size, size))
        merged = np.asarray(cv2.morphologyEx(merged, cv2.MORPH_CLOSE, kernel), dtype=np.uint8)
    count, labels_mat, stats_mat, centroids_mat = cv2.connectedComponentsWithStats(merged)
    labels = np.asarray(labels_mat, dtype=np.int32)
    stats = np.asarray(stats_mat, dtype=np.int64)
    centroids = np.asarray(centroids_mat, dtype=np.float64)
    above_labels = labels[above]
    counts = np.bincount(above_labels, minlength=count)
    peaks = np.zeros(count, dtype=np.float64)
    np.maximum.at(peaks, above_labels, magnitude[above])
    found: list[Detection] = []
    for i in range(1, count):
        area = float(counts[i]) * scale**2
        if not cfg.min_area <= area <= cfg.max_area:
            continue
        left, top, width, height = (float(v) * scale for v in stats[i, :4])
        amplitude = float(peaks[i])
        cx, cy = ((float(v) + 0.5) * scale - 0.5 for v in centroids[i])
        found.append(Detection(frame, cx, cy, left, top, width, height, area, amplitude))
    return found


def raised_threshold(
    residual: npt.NDArray[np.float64],
    samples: Sequence[NumericFrame],
    background: npt.NDArray[np.float64],
    gains: Sequence[float],
    cfg: DetectConfig,
) -> PixelThreshold:
    candidates = np.flatnonzero(np.abs(residual) > cfg.threshold)
    if candidates.size == 0:
        return cfg.threshold
    noise = temporal_noise([s.ravel()[candidates] for s in samples], background.ravel()[candidates], gains)
    threshold = np.full(residual.shape, cfg.threshold, dtype=np.float64)
    threshold.ravel()[candidates] = np.maximum(cfg.threshold, cfg.noise_factor * noise)
    return threshold


def target_blur(frame: GrayFrame, sigma: float) -> NumericFrame:
    if sigma == 0:
        return frame
    return np.asarray(cv2.GaussianBlur(frame.astype(np.float32), (0, 0), sigma), dtype=np.float32)


def detect_frames(
    frames: Iterable[GrayFrame], fps: float, cfg: DetectConfig, scale: float = 1.0
) -> dict[int, list[Detection]]:
    half = cfg.half_window(fps)
    iterator = iter(frames)
    first = next(iterator, None)
    if first is None:
        return {}
    blur = cfg.target_sigma / scale
    mask = osd_mask(first.shape[1], first.shape[0], cfg, math.ceil(BLUR_REACH * blur))
    if not mask.any():
        raise ValueError("osd_regions mask the whole frame")
    window: deque[tuple[int, NumericFrame, float]] = deque(maxlen=2 * half + 1)
    detections: dict[int, list[Detection]] = {}

    def process(target: int) -> None:
        entries = [e for e in window if abs(e[0] - target) <= half]
        current = next(e for e in entries if e[0] == target)
        sampled = entries[:: cfg.bg_step]
        background = temporal_median([e[1] for e in sampled])
        mean_level = float(np.mean([e[2] for e in sampled]))
        residual = current[1] - background - (current[2] - mean_level)
        residual[~mask] = 0
        threshold: PixelThreshold = cfg.threshold
        if cfg.noise_factor > 0 and len(sampled) >= MIN_NOISE_SAMPLES:
            gains = [e[2] - mean_level for e in sampled]
            threshold = raised_threshold(residual, [e[1] for e in sampled], background, gains, cfg)
        detections[target] = find_blobs(residual, target, cfg, scale, threshold)

    index = -1
    for index, gray in enumerate(itertools.chain([first], iterator)):
        work = target_blur(gray, blur)
        window.append((index, work, float(work[mask].mean())))
        if index - half >= 0:
            process(index - half)
    for target in range(max(0, index - half + 1), index + 1):
        process(target)
    return detections
