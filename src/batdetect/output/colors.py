from __future__ import annotations

import numpy as np

from batdetect.output.geometry import Points, resample

NEIGHBOR_SAMPLES_PER_DISTANCE = 4


def path_distance(a: Points, b: Points) -> float:
    return float(np.min(np.hypot(a[:, None, 0] - b[None, :, 0], a[:, None, 1] - b[None, :, 1])))


def assign_colors(paths: list[Points], palette_size: int, near: float) -> list[int]:
    sampled = [resample(p, near / NEIGHBOR_SAMPLES_PER_DISTANCE) for p in paths]
    colors: list[int] = []
    for i, path in enumerate(sampled):
        neighbor_colors = [colors[j] for j in range(i) if path_distance(path, sampled[j]) < near]
        ranks = [(neighbor_colors.count(c), colors.count(c), c) for c in range(palette_size)]
        colors.append(min(ranks)[2])
    return colors
