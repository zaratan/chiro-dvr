from __future__ import annotations

from pathlib import Path

import cv2
import numpy as np
import pytest

from batdetect.output.periods import Period
from batdetect.output.style import BACKGROUND_DIM, SEPARATOR, style_for
from batdetect.output.summary import compose_summary, header_lines, summary_image
from batdetect.track import Track
from batdetect.video import VideoInfo
from helpers import BACKGROUND, line

STYLE = style_for(320)
INFO = VideoInfo(30.0, 40, 320, 240, 320, 240)


def gray(width: int = 320, height: int = 240) -> np.ndarray[tuple[int, int, int], np.dtype[np.uint8]]:
    return np.full((height, width, 3), BACKGROUND, dtype=np.uint8)


def test_composition_keeps_the_video_and_adds_the_legend_on_the_right() -> None:
    image = compose_summary(gray(), [Track(1, line(0, 20, (40, 200), (10, -5)))], 30.0, ["v"], STYLE)

    assert image.shape[0] == 240
    assert image.shape[1] > 320
    assert image[5, 5].mean() == pytest.approx(BACKGROUND * BACKGROUND_DIM, abs=1)
    assert tuple(image[120, 320].tolist()) == SEPARATOR


def test_a_later_outline_does_not_cut_an_earlier_trajectory() -> None:
    crossing = [Track(1, line(0, 30, (10, 120), (10, 0))), Track(2, line(0, 24, (160, 5), (0, 10)))]

    image = compose_summary(gray(), crossing, 30.0, ["v"], STYLE)

    beside_the_crossing = image[120, 160 + STYLE.outline // 2]
    assert int(beside_the_crossing.max()) > 100


def test_header_counts_passages_and_shows_the_period() -> None:
    one = Period(0, 300, [Track(1, line(0, 3, (0, 0), (1, 0)))], alone=True)

    assert header_lines("v092", one) == ["v092", "1 passage", "0:00 - 5:00"]
    assert header_lines("v092", Period(600, 1200, [], alone=False))[1:] == ["0 passage", "10:00 - 20:00"]


def test_summary_writes_one_image_and_removes_stale_ones(tmp_path: Path) -> None:
    stale = tmp_path / "v.tracks_000m-010m.png"
    stale.write_bytes(b"old")
    table = tmp_path / "v.tracks.csv"
    table.write_text("id\n")

    written = summary_image(gray(), [Track(1, line(0, 20, (40, 200), (10, -5)))], INFO, tmp_path / "v")

    assert written == [tmp_path / "v.tracks.png"]
    assert not stale.exists()
    assert table.exists()
    assert np.asarray(cv2.imread(str(written[0]))).shape[0] == 240


def test_long_video_writes_one_image_per_period(tmp_path: Path) -> None:
    two_minutes_and_more = VideoInfo(30.0, 3700, 320, 240, 320, 240)

    written = summary_image(gray(), [], two_minutes_and_more, tmp_path / "v", span_s=60)

    assert [p.name for p in written] == ["v.tracks_000m-001m.png", "v.tracks_001m-002m.png", "v.tracks_002m-003m.png"]
    assert all(p.exists() for p in written)
