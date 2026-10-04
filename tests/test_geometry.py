from __future__ import annotations

import numpy as np
import pytest

from batdetect.output.geometry import cumulative_length, end_direction, point_at_length, resample, track_points
from batdetect.track import Track
from helpers import line


def path(*points: tuple[float, float]) -> np.ndarray[tuple[int, int], np.dtype[np.float64]]:
    return np.array(points, dtype=np.float64)


def test_track_points_keep_original_pixel_coordinates() -> None:
    assert track_points(Track(1, line(0, 3, (10.5, 20), (2, 0)))).tolist() == [[10.5, 20], [12.5, 20], [14.5, 20]]


def test_cumulative_length_adds_segment_lengths() -> None:
    assert cumulative_length(path((0, 0), (3, 4), (3, 10))).tolist() == [0, 5, 11]


def test_point_at_length_follows_distance_not_point_count() -> None:
    slow_start = path((0, 0), (1, 0), (2, 0), (3, 0), (100, 0))

    assert point_at_length(slow_start, 50) == (50, 0)


def test_single_point_path_has_zero_length_and_stays_put() -> None:
    lone = path((7, 9))

    assert cumulative_length(lone).tolist() == [0]
    assert point_at_length(lone, 5) == (7, 9)
    assert end_direction(lone, 10) is None


def test_resample_spaces_points_evenly_and_keeps_the_end() -> None:
    assert resample(path((0, 0), (10, 0)), 4).tolist() == [[0, 0], [4, 0], [8, 0], [10, 0]]


def test_end_direction_looks_back_over_the_span_not_the_last_jitter() -> None:
    jittery_end = path((0, 0), (100, 0), (100, 1))

    dx, dy = end_direction(jittery_end, 50) or (0, 0)

    assert dx == pytest.approx(1, abs=0.01)
    assert dy == pytest.approx(0.02, abs=0.01)
