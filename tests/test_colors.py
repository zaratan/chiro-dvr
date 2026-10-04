from __future__ import annotations

import itertools

import numpy as np

from batdetect.output.colors import assign_colors, path_distance
from batdetect.output.geometry import Points


def horizontal(y: float) -> Points:
    return np.array([(0, y), (100, y)], dtype=np.float64)


def test_path_distance_is_the_closest_approach() -> None:
    assert path_distance(horizontal(0), horizontal(30)) == 30


def test_neighbours_never_share_a_color_even_past_the_palette_size() -> None:
    stacked = [horizontal(10 * i) for i in range(4)]

    colors = assign_colors(stacked, palette_size=3, near=15)

    assert all(a != b for a, b in itertools.pairwise(colors))


def test_distant_tracks_spread_over_the_palette_before_reusing_colors() -> None:
    far_apart = [horizontal(1000 * i) for i in range(4)]

    assert assign_colors(far_apart, palette_size=4, near=15) == [0, 1, 2, 3]


def test_close_tracks_take_the_color_least_used_around_them() -> None:
    paths = [horizontal(0), horizontal(1000), horizontal(5)]

    assert assign_colors(paths, palette_size=2, near=15) == [0, 1, 1]
