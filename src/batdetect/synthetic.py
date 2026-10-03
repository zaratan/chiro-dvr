from __future__ import annotations

import itertools
import math
from collections.abc import Iterable, Sequence
from dataclasses import dataclass

import numpy as np

from batdetect.pipeline import ColorFrame, Track, VideoInfo

Point = tuple[float, float]

SACCADE_PERIOD = 3
FLICKER_RANGE = (0.6, 1.0)
SPEED_RANGE = (15.0, 60.0)
ACCELERATION_RANGE = (1.0, 1.8)
MAX_ATTEMPTS = 500


@dataclass(frozen=True, slots=True)
class BatClass:
    amplitude: float
    sigma: float


@dataclass(frozen=True, slots=True)
class Flight:
    controls: tuple[Point, Point, Point]
    speed: float
    acceleration: float


@dataclass(frozen=True, slots=True)
class Sampling:
    seed: int
    classes: tuple[BatClass, ...]
    per_class: int
    min_distance: float


@dataclass(frozen=True, slots=True)
class SyntheticBat:
    id: int
    start_frame: int
    positions: tuple[Point, ...]
    amplitudes: tuple[float, ...]
    amplitude: float
    sigma: float

    @property
    def end_frame(self) -> int:
        return self.start_frame + len(self.positions) - 1

    def position(self, frame: int) -> Point | None:
        k = frame - self.start_frame
        return self.positions[k] if 0 <= k < len(self.positions) else None

    def amplitude_at(self, frame: int) -> float:
        return self.amplitudes[frame - self.start_frame]


def saccade_progress(frames: int, phase: int) -> list[float]:
    if frames == 1:
        return [0.0]
    steps = [2.0 if (k + phase) % SACCADE_PERIOD == 0 else 1.0 for k in range(1, frames)]
    total = sum(steps)
    progress = [0.0]
    for step in steps:
        progress.append(progress[-1] + step / total)
    return progress


def bezier(controls: tuple[Point, Point, Point], u: float) -> Point:
    (x0, y0), (x1, y1), (x2, y2) = controls
    a, b, c = (1 - u) ** 2, 2 * (1 - u) * u, u**2
    return a * x0 + b * x1 + c * x2, a * y0 + b * y1 + c * y2


def bezier_length(controls: tuple[Point, Point, Point], samples: int = 64) -> float:
    pts = [bezier(controls, k / samples) for k in range(samples + 1)]
    return sum(math.hypot(b[0] - a[0], b[1] - a[1]) for a, b in itertools.pairwise(pts))


def make_bat(bat_id: int, start_frame: int, flight: Flight, kind: BatClass, rng: np.random.Generator) -> SyntheticBat:
    frames = max(2, math.ceil(bezier_length(flight.controls) / flight.speed))
    phase = int(rng.integers(SACCADE_PERIOD))
    progress = saccade_progress(frames, phase)
    positions = tuple(bezier(flight.controls, u**flight.acceleration) for u in progress)
    flicker = rng.uniform(*FLICKER_RANGE, size=frames)
    amplitudes = tuple(float(kind.amplitude * f) for f in flicker)
    return SyntheticBat(bat_id, start_frame, positions, amplitudes, kind.amplitude, kind.sigma)


def inject(frame: ColorFrame, bats: Iterable[SyntheticBat], frame_no: int) -> dict[int, float]:
    height, width = frame.shape[:2]
    effective: dict[int, float] = {}
    for bat in bats:
        pos = bat.position(frame_no)
        if pos is None:
            continue
        x, y = pos
        radius = math.ceil(3 * bat.sigma)
        x0, x1 = max(0, math.floor(x) - radius), min(width, math.floor(x) + radius + 1)
        y0, y1 = max(0, math.floor(y) - radius), min(height, math.floor(y) + radius + 1)
        if x0 >= x1 or y0 >= y1:
            continue
        ys, xs = np.mgrid[y0:y1, x0:x1]
        blob = bat.amplitude_at(frame_no) * np.exp(-((xs - x) ** 2 + (ys - y) ** 2) / (2 * bat.sigma**2))
        before = frame[y0:y1, x0:x1].astype(np.float32)
        after = np.clip(before + blob[..., None], 0, 255)
        frame[y0:y1, x0:x1] = after.astype(np.uint8)
        change = (after - before)[..., 0]
        effective[bat.id] = float(change.min() if bat.amplitude < 0 else change.max())
    return effective


def _side_point(rng: np.random.Generator, side: int, width: int, height: int, margin: float) -> Point:
    along = float(rng.uniform(0, 1))
    points = (
        (along * width, -margin),
        (width + margin, along * height),
        (along * width, height + margin),
        (-margin, along * height),
    )
    return points[side]


def _entry_exit(rng: np.random.Generator, width: int, height: int, margin: float) -> tuple[Point, Point]:
    first = int(rng.integers(4))
    second = (first + int(rng.integers(1, 4))) % 4
    return _side_point(rng, first, width, height, margin), _side_point(rng, second, width, height, margin)


def _too_close(bat: SyntheticBat, occupied: dict[int, list[Point]], min_distance: float) -> bool:
    for k, (x, y) in enumerate(bat.positions):
        for ox, oy in occupied.get(bat.start_frame + k, []):
            if math.hypot(x - ox, y - oy) < min_distance:
                return True
    return False


def reference_occupancy(tracks: Sequence[Track], scale: float) -> dict[int, list[Point]]:
    occupied: dict[int, list[Point]] = {}
    for track in tracks:
        for p in track.points:
            occupied.setdefault(p.frame, []).append((p.x * scale, p.y * scale))
    return occupied


def random_bats(
    sampling: Sampling, info: VideoInfo, frame_count: int, occupied: dict[int, list[Point]]
) -> list[SyntheticBat]:
    children = np.random.SeedSequence(sampling.seed).spawn(len(sampling.classes) * sampling.per_class)
    occupancy = {frame: list(points) for frame, points in occupied.items()}
    bats: list[SyntheticBat] = []
    for index, child in enumerate(children):
        kind = sampling.classes[index // sampling.per_class]
        rng = np.random.default_rng(child)
        for _ in range(MAX_ATTEMPTS):
            start, end = _entry_exit(rng, info.width, info.height, 3 * kind.sigma + 2)
            control = (float(rng.uniform(0, info.width)), float(rng.uniform(0, info.height)))
            flight = Flight(
                (start, control, end), float(rng.uniform(*SPEED_RANGE)), float(rng.uniform(*ACCELERATION_RANGE))
            )
            shape = make_bat(index, 0, flight, kind, rng)
            if len(shape.positions) >= frame_count:
                continue
            first = int(rng.integers(0, frame_count - len(shape.positions)))
            candidate = SyntheticBat(index, first, shape.positions, shape.amplitudes, kind.amplitude, kind.sigma)
            if not _too_close(candidate, occupancy, sampling.min_distance * info.scale):
                bats.append(candidate)
                for k, point in enumerate(candidate.positions):
                    occupancy.setdefault(first + k, []).append(point)
                break
    return bats
