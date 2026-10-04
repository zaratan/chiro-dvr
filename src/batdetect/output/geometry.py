from __future__ import annotations

import math

import numpy as np
import numpy.typing as npt

from batdetect.track import Track

Points = npt.NDArray[np.float64]


def track_points(track: Track) -> Points:
    return np.array([(p.x, p.y) for p in track.points], dtype=np.float64).reshape(-1, 2)


def cumulative_length(points: Points) -> Points:
    steps = np.hypot(*np.diff(points, axis=0).T)
    return np.concatenate(([0.0], np.cumsum(steps)))


def point_at_length(points: Points, length: float) -> tuple[float, float]:
    cumulative = cumulative_length(points)
    x = float(np.interp(length, cumulative, points[:, 0]))
    y = float(np.interp(length, cumulative, points[:, 1]))
    return x, y


def resample(points: Points, step: float) -> Points:
    total = float(cumulative_length(points)[-1])
    lengths = [*np.arange(0.0, total, step), total]
    return np.array([point_at_length(points, length) for length in lengths], dtype=np.float64)


def end_direction(points: Points, span: float) -> tuple[float, float] | None:
    total = float(cumulative_length(points)[-1])
    back = point_at_length(points, max(0.0, total - span))
    dx, dy = points[-1, 0] - back[0], points[-1, 1] - back[1]
    norm = math.hypot(dx, dy)
    if norm == 0:
        return None
    return dx / norm, dy / norm
