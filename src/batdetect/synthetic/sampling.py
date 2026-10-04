from __future__ import annotations

import math
from collections.abc import Sequence
from dataclasses import dataclass

import numpy as np

from batdetect.synthetic.hunting import make_hunting_bat
from batdetect.synthetic.trajectory import BatClass, Flight, Point, SyntheticBat, make_bat
from batdetect.track import Track
from batdetect.video import VideoInfo

SPEED_RANGE = (15.0, 60.0)


ACCELERATION_RANGE = (1.0, 1.8)


MAX_ATTEMPTS = 500


@dataclass(frozen=True, slots=True)
class Sampling:
    seed: int
    classes: tuple[BatClass, ...]
    per_class: int
    min_distance: float


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


def draw_shape(index: int, kind: BatClass, info: VideoInfo, rng: np.random.Generator) -> SyntheticBat:
    if kind.motion != "pass":
        return make_hunting_bat(index, 0, kind, info, rng)
    start, end = _entry_exit(rng, info.width, info.height, 3 * kind.sigma + 2)
    control = (float(rng.uniform(0, info.width)), float(rng.uniform(0, info.height)))
    flight = Flight((start, control, end), float(rng.uniform(*SPEED_RANGE)), float(rng.uniform(*ACCELERATION_RANGE)))
    return make_bat(index, 0, flight, kind, rng)


def reference_occupancy(tracks: Sequence[Track]) -> dict[int, list[Point]]:
    occupied: dict[int, list[Point]] = {}
    for track in tracks:
        for p in track.points:
            occupied.setdefault(p.frame, []).append((p.x, p.y))
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
            shape = draw_shape(index, kind, info, rng)
            if len(shape.positions) >= frame_count:
                continue
            first = int(rng.integers(0, frame_count - len(shape.positions)))
            candidate = SyntheticBat(
                index, first, shape.positions, shape.amplitudes, kind.amplitude, kind.sigma, kind.motion
            )
            if not _too_close(candidate, occupancy, sampling.min_distance):
                bats.append(candidate)
                for k, point in enumerate(candidate.positions):
                    occupancy.setdefault(first + k, []).append(point)
                break
    return bats
