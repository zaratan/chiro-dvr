from __future__ import annotations

import pytest

from batdetect.output.timefmt import clip_name, format_clock, format_duration, format_time
from batdetect.track import Track
from helpers import line


@pytest.mark.parametrize(
    ("seconds", "expected"),
    [(0, "0:00.00"), (7.53, "0:07.53"), (59.996, "1:00.00"), (119.997, "2:00.00"), (237.4, "3:57.40")],
)
def test_format_time_never_shows_sixty_seconds(seconds: float, expected: str) -> None:
    assert format_time(seconds) == expected


def test_clip_name_sorts_by_id_and_carries_start_time() -> None:
    track = Track(7, line(7122, 5, (0, 0), (1, 0)))

    assert clip_name(track, 30.0) == "07_3m57s40.mp4"


def test_clock_drops_hundredths_and_rounds_down_to_the_second() -> None:
    assert [format_clock(s) for s in (0, 7.99, 237.32, 600)] == ["0:00", "0:07", "3:57", "10:00"]


def test_duration_under_a_minute_is_in_seconds() -> None:
    assert format_duration(44.2) == "45 s"


def test_duration_over_a_minute_gives_minutes_and_seconds() -> None:
    assert format_duration(77.8) == "1 min 18 s"


def test_one_damaged_frame_never_reads_as_zero_seconds() -> None:
    assert format_duration(1 / 30) == "1 s"
