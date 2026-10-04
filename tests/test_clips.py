from __future__ import annotations

from pathlib import Path

import pytest

from batdetect.output.clips import ClipWindow, clip_windows, in_passes
from batdetect.track import Track
from helpers import line

SPLIT = Path("split")


def window(first: int, last: int) -> ClipWindow:
    return ClipWindow(first, last, SPLIT / f"{first}-{last}.mp4")


def most_simultaneous(windows: list[ClipWindow]) -> int:
    return max(sum(w.first <= f <= w.last for w in windows) for w in windows for f in (w.first, w.last))


def test_margin_is_counted_in_frames_and_clamped_at_the_start_only() -> None:
    early = Track(1, line(5, 10, (20, 20), (5, 0)))
    late = Track(2, line(100, 10, (20, 20), (5, 0)))

    windows = clip_windows([early, late], fps=30.0, margin_s=0.5, split_dir=SPLIT)

    assert [(w.first, w.last) for w in windows] == [(0, 29), (85, 124)]


def test_clip_is_named_after_its_track_and_start() -> None:
    track = Track(3, line(45, 10, (20, 20), (5, 0)))

    assert clip_windows([track], fps=30.0, margin_s=0.0, split_dir=SPLIT)[0].path == SPLIT / "03_0m01s50.mp4"


def test_windows_that_never_overlap_share_a_single_pass() -> None:
    windows = [window(0, 9), window(10, 19), window(20, 29)]

    assert in_passes(windows, max_writers=1) == [windows]


def test_windows_touching_on_one_frame_need_two_writers() -> None:
    assert in_passes([window(0, 10), window(10, 20)], max_writers=1) == [[window(0, 10)], [window(10, 20)]]


def test_passes_never_hold_more_simultaneous_windows_than_allowed() -> None:
    windows = [window(i, i + 20) for i in range(9)] + [window(40, 50)]

    passes = in_passes(windows, max_writers=4)

    assert sorted(w.first for batch in passes for w in batch) == sorted(w.first for w in windows)
    assert all(most_simultaneous(batch) <= 4 for batch in passes)
    assert len(passes) == 3


def test_no_window_gives_no_pass() -> None:
    assert in_passes([], max_writers=4) == []


def test_at_least_one_writer_is_needed() -> None:
    with pytest.raises(ValueError, match="max_writers"):
        in_passes([window(0, 1)], max_writers=0)
