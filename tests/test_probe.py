from __future__ import annotations

import os
import signal
import subprocess
from pathlib import Path

import cv2
import pytest

from batdetect.damage import Probe, check_frame_count
from batdetect.probe import probing
from batdetect.video import VideoError, read_frames
from helpers import damaged_video, finishes, requires_ffmpeg


class DetectionFailedError(Exception):
    pass


def probed(video: Path) -> Probe:
    with probing(video) as outcome:
        pass
    return outcome.probe


def opencv_frame_count(video: Path) -> int:
    cap = cv2.VideoCapture(str(video))
    try:
        return sum(1 for _ in read_frames(cap))
    finally:
        cap.release()


@requires_ffmpeg
def test_finds_exactly_the_frames_whose_slice_was_cut(tmp_path: Path) -> None:
    result = probed(damaged_video(tmp_path / "cut.mp4", {20, 33, 47, 104}))

    assert result.frame_count == 120
    assert result.error_frames == (20, 33, 47, 104)
    assert result.key_frames == (0, 15, 30, 45, 60, 75, 90, 105)


@requires_ffmpeg
def test_damage_on_a_key_frame_is_reported(tmp_path: Path) -> None:
    assert probed(damaged_video(tmp_path / "key.mp4", {90})).error_frames == (90,)


@requires_ffmpeg
def test_clean_video_has_no_damaged_frame(tmp_path: Path) -> None:
    result = probed(damaged_video(tmp_path / "clean.mp4", set()))

    assert (result.frame_count, result.error_frames, result.gap_frames) == (120, (), ())


@requires_ffmpeg
def test_lost_frame_makes_ffprobe_and_opencv_counts_differ(tmp_path: Path) -> None:
    video = damaged_video(tmp_path / "lost.mp4", {47}, fraction=1.0)
    probed_count, read_count = probed(video).frame_count, opencv_frame_count(video)

    assert read_count < probed_count < 120
    with pytest.raises(VideoError, match="frame numbers do not match"):
        check_frame_count(probed_count, read_count)


@requires_ffmpeg
def test_probe_is_killed_when_the_block_fails(tmp_path: Path) -> None:
    never_written = tmp_path / "never_written.mp4"
    os.mkfifo(never_written)
    killed: list[subprocess.Popen[bytes]] = []

    def detect_and_fail() -> None:
        with probing(never_written) as outcome:
            killed.append(outcome.process)
            raise DetectionFailedError

    def fail_during_detection() -> None:
        with pytest.raises(DetectionFailedError):
            detect_and_fail()

    assert finishes(fail_during_detection)
    assert killed[0].returncode == -signal.SIGKILL


@requires_ffmpeg
def test_probe_is_only_read_after_its_block(tmp_path: Path) -> None:
    with probing(damaged_video(tmp_path / "v.mp4", set())) as outcome, pytest.raises(RuntimeError):
        _ = outcome.probe


@requires_ffmpeg
def test_probe_failure_is_a_video_error_that_gives_ffprobe_reason(tmp_path: Path) -> None:
    not_a_video = tmp_path / "notes.mp4"
    not_a_video.write_text("not a video")

    with pytest.raises(VideoError, match=r"ffprobe failed .*Invalid data"), probing(not_a_video):
        pass


def test_missing_ffprobe_is_an_os_error(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("PATH", "")

    with pytest.raises(FileNotFoundError), probing(tmp_path / "v.mp4"):
        pass
