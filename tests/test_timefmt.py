from __future__ import annotations

import pytest

from batdetect.output.timefmt import clip_name, format_time
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
