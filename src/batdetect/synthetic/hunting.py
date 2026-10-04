from __future__ import annotations

import math
from collections.abc import Iterator

import numpy as np

from batdetect.synthetic.trajectory import (
    FLICKER_RANGE,
    SACCADE_PERIOD,
    BatClass,
    Point,
    SyntheticBat,
    saccade_progress,
)
from batdetect.video import VideoInfo

CRUISE_SPEED_RANGE = (8.0, 30.0)
APEX_SPEED_RANGE = (0.25, 0.5)
TURN_ANGLE_RANGE = (math.pi / 4, math.pi)
APEX_TURN_RATE_RANGE = (0.15, 0.4)
MIN_TURN_FRAMES = 4
MAX_TURN_FRAMES = 60
MAX_DURATION_S = 4.0
FIRST_STRAIGHT_S = (0.1, 0.3)
STRAIGHT_S = {"hunt": (0.2, 1.0), "circle": (0.1, 0.3)}
STRAIGHT_DRIFT = 0.05
ENTRY_SPREAD = math.pi / 4

type Step = tuple[float, float]


def turn_profile(cruise: float, apex_ratio: float, apex_rate: float, angle: float) -> list[Step]:
    ratios: list[float] = []
    rates: list[float] = []
    for frames in range(MIN_TURN_FRAMES, MAX_TURN_FRAMES + 1):
        ratios = [1 - (1 - apex_ratio) * math.sin(math.pi * (k + 0.5) / frames) ** 2 for k in range(frames)]
        rates = [apex_rate * apex_ratio / r for r in ratios]
        if sum(rates) >= angle:
            break
    scale = min(1.0, angle / sum(rates))
    return [(cruise * r, w * scale) for r, w in zip(ratios, rates, strict=True)]


def straight(rng: np.random.Generator, cruise: float, seconds: float, fps: float) -> list[Step]:
    frames = max(1, round(seconds * fps))
    return [(cruise, float(rng.uniform(-STRAIGHT_DRIFT, STRAIGHT_DRIFT))) for _ in range(frames)]


def maneuvers(rng: np.random.Generator, kind: BatClass, cruise: float, fps: float) -> Iterator[Step]:
    yield from straight(rng, cruise, float(rng.uniform(*FIRST_STRAIGHT_S)), fps)
    while True:
        turn = turn_profile(
            cruise,
            float(rng.uniform(*APEX_SPEED_RANGE)),
            float(rng.uniform(*APEX_TURN_RATE_RANGE)),
            float(rng.uniform(*TURN_ANGLE_RANGE)),
        )
        sign = float(rng.choice((-1.0, 1.0)))
        yield from ((speed, sign * rate) for speed, rate in turn)
        yield from straight(rng, cruise, float(rng.uniform(*STRAIGHT_S[kind.motion])), fps)


def entry(rng: np.random.Generator, info: VideoInfo, margin: float) -> tuple[Point, float]:
    side = int(rng.integers(4))
    along = float(rng.uniform(0, 1))
    starts = (
        (along * info.width, -margin),
        (info.width + margin, along * info.height),
        (along * info.width, info.height + margin),
        (-margin, along * info.height),
    )
    x, y = starts[side]
    heading = math.atan2(info.height / 2 - y, info.width / 2 - x) + float(rng.uniform(-ENTRY_SPREAD, ENTRY_SPREAD))
    return (x, y), heading


def hunting_path(rng: np.random.Generator, kind: BatClass, info: VideoInfo) -> list[Point]:
    margin = 3 * kind.sigma + 2
    (x, y), heading = entry(rng, info, margin)
    points = [(x, y)]
    entered = False
    limit = round(MAX_DURATION_S * info.fps)
    for speed, turn in maneuvers(rng, kind, float(rng.uniform(*CRUISE_SPEED_RANGE)), info.fps):
        heading += turn
        x, y = x + speed * math.cos(heading), y + speed * math.sin(heading)
        points.append((x, y))
        inside = 0 <= x <= info.width and 0 <= y <= info.height
        entered = entered or inside
        outside = not (-margin <= x <= info.width + margin and -margin <= y <= info.height + margin)
        if len(points) >= limit or (entered and outside):
            break
    return points


def with_saccade(points: list[Point], phase: int) -> tuple[Point, ...]:
    frames = np.arange(len(points), dtype=np.float64)
    times = np.array(saccade_progress(len(points), phase)) * (len(points) - 1)
    xs = np.interp(times, frames, [p[0] for p in points])
    ys = np.interp(times, frames, [p[1] for p in points])
    return tuple((float(x), float(y)) for x, y in zip(xs, ys, strict=True))


def make_hunting_bat(
    bat_id: int, start_frame: int, kind: BatClass, info: VideoInfo, rng: np.random.Generator
) -> SyntheticBat:
    path = hunting_path(rng, kind, info)
    positions = with_saccade(path, int(rng.integers(SACCADE_PERIOD)))
    flicker = rng.uniform(*FLICKER_RANGE, size=len(positions))
    amplitudes = tuple(float(kind.amplitude * f) for f in flicker)
    return SyntheticBat(bat_id, start_frame, positions, amplitudes, kind.amplitude, kind.sigma, kind.motion)
