from __future__ import annotations

import itertools
import math
from dataclasses import dataclass
from typing import Literal

import numpy as np

Point = tuple[float, float]

type Motion = Literal["pass", "hunt", "circle"]
MOTIONS: tuple[Motion, ...] = ("pass", "hunt", "circle")


SACCADE_PERIOD = 3


FLICKER_RANGE = (0.6, 1.0)


@dataclass(frozen=True, slots=True)
class BatClass:
    amplitude: float
    sigma: float
    motion: Motion = "pass"


@dataclass(frozen=True, slots=True)
class Flight:
    controls: tuple[Point, Point, Point]
    speed: float
    acceleration: float


@dataclass(frozen=True, slots=True)
class SyntheticBat:
    id: int
    start_frame: int
    positions: tuple[Point, ...]
    amplitudes: tuple[float, ...]
    amplitude: float
    sigma: float
    motion: Motion = "pass"

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
    return SyntheticBat(bat_id, start_frame, positions, amplitudes, kind.amplitude, kind.sigma, kind.motion)
