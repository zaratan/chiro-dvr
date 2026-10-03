from __future__ import annotations

import itertools
import math
from collections import deque
from collections.abc import Iterable, Iterator
from dataclasses import dataclass, field
from pathlib import Path

import cv2
import numpy as np
import numpy.typing as npt

GrayFrame = npt.NDArray[np.uint8]
ColorFrame = npt.NDArray[np.uint8]

MIN_BACKGROUND_FRAMES = 3
MIN_WORK_WIDTH = 16


class VideoError(Exception):
    pass


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


SYMBION_OSD = (Region(0.0, 0.0, 1.0, 0.06), Region(0.30, 0.89, 0.56, 0.98))


@dataclass(frozen=True, slots=True)
class DetectConfig:
    threshold: float = 25
    min_area: int = 2
    max_area: int = 300
    bg_window_s: float = 1.0
    bg_step: int = 3
    osd_top: float = 0.07
    osd_bottom: float = 0.10
    merge_radius: int = 2
    work_width: int = 480

    def __post_init__(self) -> None:
        if self.threshold <= 0:
            raise ValueError("threshold must be > 0")
        if not 1 <= self.min_area <= self.max_area:
            raise ValueError("expected 1 <= min_area <= max_area")
        if self.bg_window_s <= 0:
            raise ValueError("bg_window_s must be > 0")
        if self.merge_radius < 0:
            raise ValueError("merge_radius must be >= 0")
        if self.bg_step < 1:
            raise ValueError("bg_step must be >= 1")
        if not (0 <= self.osd_top < 1 and 0 <= self.osd_bottom < 1 and self.osd_top + self.osd_bottom < 1):
            raise ValueError("osd_top and osd_bottom must leave part of the frame visible")
        if self.work_width < MIN_WORK_WIDTH:
            raise ValueError(f"work_width must be >= {MIN_WORK_WIDTH}")

    def half_window(self, fps: float) -> int:
        half = round(self.bg_window_s * fps / 2)
        if 2 * half + 1 < MIN_BACKGROUND_FRAMES:
            raise ValueError(f"bg_window_s={self.bg_window_s} covers fewer than {MIN_BACKGROUND_FRAMES} frames")
        return half


@dataclass(frozen=True, slots=True)
class TrackConfig:
    max_jump: float = 40
    max_gap: int = 6
    min_hits: int = 5
    min_travel: float = 15
    twin_distance: float = 12

    def __post_init__(self) -> None:
        if self.max_jump <= 0:
            raise ValueError("max_jump must be > 0")
        if self.max_gap < 1:
            raise ValueError("max_gap must be >= 1")
        if self.min_hits < 1:
            raise ValueError("min_hits must be >= 1")
        if self.min_travel < 0:
            raise ValueError("min_travel must be >= 0")
        if self.twin_distance < 0:
            raise ValueError("twin_distance must be >= 0")


@dataclass(frozen=True, slots=True)
class Detection:
    frame: int
    x: float
    y: float
    left: int
    top: int
    width: int
    height: int
    area: int
    amplitude: float


@dataclass(frozen=True, slots=True)
class VideoInfo:
    fps: float
    frame_count: int
    width: int
    height: int
    work_width: int
    work_height: int

    @property
    def scale(self) -> float:
        return self.width / self.work_width


@dataclass(slots=True)
class Track:
    id: int
    points: list[Detection] = field(default_factory=list[Detection])

    @property
    def first(self) -> Detection:
        return self.points[0]

    @property
    def last(self) -> Detection:
        return self.points[-1]

    def predicted(self, frame: int) -> tuple[float, float]:
        last = self.last
        if len(self.points) == 1:
            return last.x, last.y
        prev = self.points[-2]
        dt = last.frame - prev.frame
        ahead = frame - last.frame
        return last.x + (last.x - prev.x) / dt * ahead, last.y + (last.y - prev.y) / dt * ahead

    def chord(self) -> float:
        return math.hypot(self.last.x - self.first.x, self.last.y - self.first.y)

    def path_length(self) -> float:
        return sum(math.hypot(b.x - a.x, b.y - a.y) for a, b in zip(self.points, self.points[1:], strict=False))


def open_video(path: Path, work_width: int) -> tuple[cv2.VideoCapture, VideoInfo]:
    cap = cv2.VideoCapture(str(path))
    if not cap.isOpened():
        raise VideoError(f"cannot open {path}")
    width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    fps = cap.get(cv2.CAP_PROP_FPS)
    if width <= 0 or height <= 0 or fps <= 0:
        cap.release()
        raise VideoError(f"{path} has no readable video stream")
    work_height = max(1, round(work_width * height / width))
    info = VideoInfo(fps, int(cap.get(cv2.CAP_PROP_FRAME_COUNT)), width, height, work_width, work_height)
    return cap, info


def read_frames(cap: cv2.VideoCapture) -> Iterator[ColorFrame]:
    while True:
        ok, frame = cap.read()
        if not ok:
            return
        yield np.asarray(frame, dtype=np.uint8)


def to_work_gray(frame: ColorFrame, info: VideoInfo) -> GrayFrame:
    small = cv2.resize(frame, (info.work_width, info.work_height), interpolation=cv2.INTER_AREA)
    return np.asarray(cv2.cvtColor(small, cv2.COLOR_BGR2GRAY), dtype=np.uint8)


def read_gray_frames(cap: cv2.VideoCapture, info: VideoInfo) -> Iterator[GrayFrame]:
    return (to_work_gray(frame, info) for frame in read_frames(cap))


def osd_mask(width: int, height: int, cfg: DetectConfig) -> npt.NDArray[np.bool_]:
    mask = np.ones((height, width), dtype=np.bool_)
    mask[: int(height * cfg.osd_top)] = False
    mask[height - int(height * cfg.osd_bottom) :] = False
    return mask


def find_blobs(residual: npt.NDArray[np.float64], frame: int, cfg: DetectConfig) -> list[Detection]:
    above = np.abs(residual) > cfg.threshold
    merged = above.astype(np.uint8)
    if cfg.merge_radius > 0:
        size = 2 * cfg.merge_radius + 1
        kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (size, size))
        merged = np.asarray(cv2.morphologyEx(merged, cv2.MORPH_CLOSE, kernel), dtype=np.uint8)
    count, labels_mat, stats_mat, centroids_mat = cv2.connectedComponentsWithStats(merged)
    labels = np.asarray(labels_mat)
    stats = np.asarray(stats_mat, dtype=np.int64)
    centroids = np.asarray(centroids_mat, dtype=np.float64)
    found: list[Detection] = []
    for i in range(1, count):
        pixels = above & (labels == i)
        area = int(pixels.sum())
        if not cfg.min_area <= area <= cfg.max_area:
            continue
        left, top, width, height = (int(v) for v in stats[i, :4])
        amplitude = float(np.abs(residual[pixels]).max())
        cx, cy = (float(v) for v in centroids[i])
        found.append(Detection(frame, cx, cy, left, top, width, height, area, amplitude))
    return found


def detect_frames(frames: Iterable[GrayFrame], fps: float, cfg: DetectConfig) -> dict[int, list[Detection]]:
    half = cfg.half_window(fps)
    iterator = iter(frames)
    first = next(iterator, None)
    if first is None:
        return {}
    mask = osd_mask(first.shape[1], first.shape[0], cfg)
    window: deque[tuple[int, npt.NDArray[np.int16], float]] = deque(maxlen=2 * half + 1)
    detections: dict[int, list[Detection]] = {}

    def process(target: int) -> None:
        entries = [e for e in window if abs(e[0] - target) <= half]
        current = next(e for e in entries if e[0] == target)
        sampled = entries[:: cfg.bg_step]
        background = np.median(np.stack([e[1] for e in sampled]), axis=0)
        offset = current[2] - float(np.mean([e[2] for e in sampled]))
        residual = current[1] - background - offset
        residual[~mask] = 0
        detections[target] = find_blobs(residual, target, cfg)

    index = -1
    for index, gray in enumerate(itertools.chain([first], iterator)):
        window.append((index, gray.astype(np.int16), float(gray[mask].mean())))
        if index - half >= 0:
            process(index - half)
    for target in range(max(0, index - half + 1), index + 1):
        process(target)
    return detections


def _match(active: list[Track], candidates: list[Detection], frame: int, max_jump: float) -> list[Detection]:
    pairs: list[tuple[bool, float, int, int]] = []
    for ti, track in enumerate(active):
        px, py = track.predicted(frame)
        has_velocity = len(track.points) > 1
        for di, det in enumerate(candidates):
            dist = math.hypot(det.x - px, det.y - py)
            if dist <= max_jump:
                pairs.append((not has_velocity, dist, ti, di))
    used_tracks: set[int] = set()
    used_dets: set[int] = set()
    for _, _, ti, di in sorted(pairs):
        if ti in used_tracks or di in used_dets:
            continue
        active[ti].points.append(candidates[di])
        used_tracks.add(ti)
        used_dets.add(di)
    return [d for di, d in enumerate(candidates) if di not in used_dets]


def track_detections(detections: dict[int, list[Detection]], cfg: TrackConfig) -> list[Track]:
    active: list[Track] = []
    finished: list[Track] = []
    for frame in sorted(detections):
        finished.extend(t for t in active if frame - t.last.frame - 1 > cfg.max_gap)
        active = [t for t in active if frame - t.last.frame - 1 <= cfg.max_gap]
        unmatched = _match(active, detections[frame], frame, cfg.max_jump)
        active.extend(Track(0, [d]) for d in unmatched)
    finished.extend(active)
    kept = [
        t
        for t in merge_twins(finished, cfg.twin_distance)
        if len(t.points) >= cfg.min_hits and t.chord() >= cfg.min_travel
    ]
    kept.sort(key=lambda t: (t.first.frame, t.first.x))
    for number, track in enumerate(kept, 1):
        track.id = number
    return kept


def are_twins(a: Track, b: Track, max_distance: float) -> bool:
    if a.last.frame < b.first.frame or b.last.frame < a.first.frame:
        return False
    by_frame = {p.frame: p for p in a.points}
    common = [(by_frame[p.frame], p) for p in b.points if p.frame in by_frame]
    return bool(common) and all(math.hypot(pa.x - pb.x, pa.y - pb.y) <= max_distance for pa, pb in common)


def combine(a: Detection, b: Detection) -> Detection:
    left, top = min(a.left, b.left), min(a.top, b.top)
    right = max(a.left + a.width, b.left + b.width)
    bottom = max(a.top + a.height, b.top + b.height)
    area = a.area + b.area
    x = (a.x * a.area + b.x * b.area) / area
    y = (a.y * a.area + b.y * b.area) / area
    return Detection(a.frame, x, y, left, top, right - left, bottom - top, area, max(a.amplitude, b.amplitude))


def merge_twins(tracks: list[Track], max_distance: float) -> list[Track]:
    if max_distance <= 0:
        return tracks
    ordered = sorted(tracks, key=lambda t: t.first.frame)
    parent = list(range(len(ordered)))

    def root(i: int) -> int:
        while parent[i] != i:
            parent[i] = parent[parent[i]]
            i = parent[i]
        return i

    for i, a in enumerate(ordered):
        for j in range(i + 1, len(ordered)):
            b = ordered[j]
            if b.first.frame > a.last.frame:
                break
            if are_twins(a, b, max_distance):
                parent[root(j)] = root(i)

    groups: dict[int, list[Track]] = {}
    for i, track in enumerate(ordered):
        groups.setdefault(root(i), []).append(track)
    merged: list[Track] = []
    for members in groups.values():
        by_frame: dict[int, Detection] = {}
        for det in (p for t in members for p in t.points):
            by_frame[det.frame] = combine(by_frame[det.frame], det) if det.frame in by_frame else det
        merged.append(Track(0, [by_frame[f] for f in sorted(by_frame)]))
    return merged
