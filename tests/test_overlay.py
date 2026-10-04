from __future__ import annotations

from batdetect.output.overlay import filled_points, trail_points
from batdetect.track import Track
from helpers import detection


def test_gaps_are_filled_by_linear_interpolation_marked_as_such() -> None:
    track = Track(1, [detection(10, 0, 0), detection(13, 30, 60)])

    filled = filled_points(track)

    assert [(d.frame, round(d.x), round(d.y), interpolated) for d, interpolated in filled] == [
        (10, 0, 0, False),
        (11, 10, 20, True),
        (12, 20, 40, True),
        (13, 30, 60, False),
    ]


def test_filling_gaps_leaves_the_detections_untouched() -> None:
    track = Track(1, [detection(10, 0, 0), detection(13, 30, 60)])

    filled_points(track)

    assert [d.frame for d in track.points] == [10, 13]


def test_trail_follows_the_interpolated_position_instead_of_stopping_at_the_last_detection() -> None:
    track = Track(1, [detection(10, 0, 0), detection(11, 10, 0), detection(14, 40, 0)])
    current = filled_points(track)[3][0]

    assert trail_points(track, current, trail_frames=30) == [(0, 0), (10, 0), (30, 0)]


def test_trail_on_a_detection_ends_on_it_without_repeating_it() -> None:
    track = Track(1, [detection(10, 0, 0), detection(11, 10, 0), detection(14, 40, 0)])

    assert trail_points(track, track.points[2], trail_frames=30) == [(0, 0), (10, 0), (40, 0)]


def test_trail_keeps_only_the_last_trail_frames() -> None:
    track = Track(1, [detection(10, 0, 0), detection(11, 10, 0), detection(14, 40, 0)])

    assert trail_points(track, track.points[2], trail_frames=3) == [(10, 0), (40, 0)]
