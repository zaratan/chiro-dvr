from __future__ import annotations

import math

import numpy as np

from batdetect.output.geometry import Points
from batdetect.output.placement import CROSSING_REACH, MARKER_GAP, place_markers

SIZE = (400, 300)
RADIUS = 10


def path(*points: tuple[float, float]) -> Points:
    return np.array(points, dtype=np.float64)


def test_marker_stays_whole_inside_the_frame_for_a_track_along_the_edge() -> None:
    ((x, y),) = place_markers([path((399, 0), (399, 299))], SIZE, RADIUS, [], 0)

    inset = RADIUS + MARKER_GAP
    assert inset <= x <= SIZE[0] - inset
    assert inset <= y <= SIZE[1] - inset


def test_markers_of_close_parallel_tracks_do_not_overlap() -> None:
    paths = [path((50, 100), (350, 100)), path((50, 105), (350, 105))]

    a, b = place_markers(paths, SIZE, RADIUS, [], 0)

    assert math.dist(a, b) >= 2 * RADIUS


def test_marker_moves_off_a_crossing_that_sits_on_its_preferred_spot() -> None:
    crossing_at_fifth = [path((50, 150), (350, 150)), path((110, 50), (110, 250))]

    first, _ = place_markers(crossing_at_fifth, SIZE, RADIUS, [], 0)

    assert math.dist(first, (110, 150)) >= CROSSING_REACH * RADIUS


def test_marker_sits_near_a_quarter_of_the_track_rather_than_at_its_very_start() -> None:
    ((x, _),) = place_markers([path((0, 150), (400, 150))], SIZE, RADIUS, [], 0)

    assert x == 80


def test_marker_keeps_clear_of_arrow_heads() -> None:
    ((x, y),) = place_markers([path((0, 150), (400, 150))], SIZE, RADIUS, [(100, 150)], 40)

    assert math.dist((x, y), (100, 150)) >= 40


def test_single_point_track_gets_its_marker_on_that_point() -> None:
    assert place_markers([path((60, 70))], SIZE, RADIUS, [], 0) == [(60, 70)]


def test_superimposed_tracks_get_separate_markers_the_same_way_every_time() -> None:
    same = [path((50, 150), (350, 150)), path((50, 150), (350, 150))]

    first = place_markers(same, SIZE, RADIUS, [], 0)

    assert first == place_markers(same, SIZE, RADIUS, [], 0)
    assert math.dist(*first) >= 2 * RADIUS
