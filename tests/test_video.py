from __future__ import annotations

from pathlib import Path

import pytest

from batdetect.video import VideoError, open_video, read_frames, read_gray_frames, to_work_gray
from helpers import small_video


def test_video_info_reports_size_rate_and_proportional_work_size(tmp_path: Path) -> None:
    cap, info = open_video(small_video(tmp_path / "v.mp4"), work_width=160)
    cap.release()

    assert (info.width, info.height, info.fps) == (320, 240, 30)
    assert (info.work_width, info.work_height) == (160, 120)
    assert info.scale == 2


def test_work_width_never_exceeds_the_video_width(tmp_path: Path) -> None:
    cap, info = open_video(small_video(tmp_path / "v.mp4"), work_width=1000)
    cap.release()

    assert info.work_width == 320
    assert info.scale == 1


def test_unreadable_file_raises_a_video_error(tmp_path: Path) -> None:
    broken = tmp_path / "broken.mp4"
    broken.write_bytes(b"not a video")

    with pytest.raises(VideoError, match="cannot open"):
        open_video(broken, 480)


def test_every_frame_is_read_in_full_colour(tmp_path: Path) -> None:
    cap, _ = open_video(small_video(tmp_path / "v.mp4", frames=25), 480)

    frames = list(read_frames(cap))
    cap.release()

    assert len(frames) == 25
    assert frames[0].shape == (240, 320, 3)


def test_gray_frames_are_reduced_to_the_work_size(tmp_path: Path) -> None:
    cap, info = open_video(small_video(tmp_path / "v.mp4", frames=5), work_width=160)

    grays = list(read_gray_frames(cap, info))
    cap.release()

    assert [g.shape for g in grays] == [(120, 160)] * 5


def test_native_work_size_skips_the_reduction(tmp_path: Path) -> None:
    cap, info = open_video(small_video(tmp_path / "v.mp4", frames=1), work_width=320)
    frame = next(read_frames(cap))
    cap.release()

    assert to_work_gray(frame, info).shape == (240, 320)
