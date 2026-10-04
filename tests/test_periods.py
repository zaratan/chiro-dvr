from __future__ import annotations

from batdetect.output.periods import split_by_period
from batdetect.track import Track
from batdetect.video import VideoInfo
from helpers import line

FPS = 10.0


def info(seconds: float) -> VideoInfo:
    return VideoInfo(FPS, round(seconds * FPS), 320, 240, 320, 240)


def starting_at(track_id: int, seconds: float) -> Track:
    return Track(track_id, line(round(seconds * FPS), 3, (10, 10), (1, 0)))


def test_short_video_gives_a_single_image_without_suffix() -> None:
    (period,) = split_by_period([starting_at(1, 5)], info(300), span_s=600)

    assert (period.start_s, period.end_s, period.suffix) == (0, 300, "")
    assert len(period.tracks) == 1


def test_video_exactly_one_span_long_still_gives_a_single_image() -> None:
    assert len(split_by_period([], info(600), span_s=600)) == 1


def test_long_video_is_split_and_the_last_period_ends_with_the_video() -> None:
    periods = split_by_period([], info(1400), span_s=600)

    assert [(p.start_s, p.end_s) for p in periods] == [(0, 600), (600, 1200), (1200, 1400)]
    assert [p.suffix for p in periods] == ["_000m-010m", "_010m-020m", "_020m-024m"]


def test_track_goes_to_the_period_where_it_starts_even_on_the_boundary() -> None:
    periods = split_by_period([starting_at(1, 599.9), starting_at(2, 600)], info(1400), span_s=600)

    assert [[t.id for t in p.tracks] for p in periods] == [[1], [2], []]
