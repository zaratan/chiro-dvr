from __future__ import annotations

from batdetect.output.overlay import filled_points
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
