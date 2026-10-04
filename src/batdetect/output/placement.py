from __future__ import annotations

import math

import numpy as np

from batdetect.output.geometry import Points, cumulative_length, point_at_length, resample

CANDIDATE_FRACTIONS = tuple(i / 10 for i in range(1, 10))
PREFERRED_FRACTION = 0.25
START_BIAS = 2.0
CROSSING_REACH = 2.0
MARKER_GAP = 2

Center = tuple[float, float]


def clamp(point: Center, size: tuple[int, int], radius: int) -> Center:
    width, height = size
    inset = radius + MARKER_GAP
    return min(max(point[0], inset), width - inset), min(max(point[1], inset), height - inset)


def candidates(path: Points, size: tuple[int, int], radius: int) -> list[tuple[float, Center]]:
    total = float(cumulative_length(path)[-1])
    fractions = CANDIDATE_FRACTIONS if total > 0 else (0.0,)
    return [(f, clamp(point_at_length(path, f * total), size, radius)) for f in fractions]


def crossing_cost(center: Center, others: list[Points], reach: float) -> int:
    return sum(int(np.count_nonzero(np.hypot(o[:, 0] - center[0], o[:, 1] - center[1]) < reach)) for o in others)


def collisions(center: Center, placed: list[Center], radius: int, keep_clear: list[Center], clear: float) -> int:
    markers = sum(math.dist(center, c) < 2 * radius + MARKER_GAP for c in placed)
    return markers + sum(math.dist(center, c) < clear for c in keep_clear)


def place_markers(
    paths: list[Points], size: tuple[int, int], radius: int, keep_clear: list[Center], clear_radius: float
) -> list[Center]:
    sampled = [resample(p, radius / 2) for p in paths]
    order = sorted(range(len(paths)), key=lambda i: float(cumulative_length(paths[i])[-1]))
    placed: dict[int, Center] = {}
    for i in order:
        others = [s for j, s in enumerate(sampled) if j != i]
        scored = [
            (
                collisions(center, list(placed.values()), radius, keep_clear, clear_radius),
                crossing_cost(center, others, CROSSING_REACH * radius) + START_BIAS * abs(f - PREFERRED_FRACTION),
                f,
                center,
            )
            for f, center in candidates(paths[i], size, radius)
        ]
        placed[i] = min(scored)[3]
    return [placed[i] for i in range(len(paths))]
