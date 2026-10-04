from __future__ import annotations

import subprocess
from pathlib import Path

import pytest

from batdetect.output.clips import split_clips
from batdetect.output.config import X264, RenderConfig
from batdetect.track import Track
from batdetect.video import VideoError, VideoInfo
from helpers import EVERY_ENCODER, line, requires_ffmpeg, small_video

INFO = VideoInfo(fps=30.0, frame_count=90, width=320, height=240, work_width=320, work_height=240)


def duration_of(path: Path) -> float:
    probe = ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(path)]
    return float(subprocess.run(probe, capture_output=True, text=True, check=True).stdout)


@requires_ffmpeg
@pytest.mark.parametrize("encoder", EVERY_ENCODER)
def test_one_clip_per_track_with_margins_clamped_to_the_video(tmp_path: Path, encoder: str) -> None:
    video = small_video(tmp_path / "v.mp4", frames=90)
    track = Track(1, line(0, 10, (20, 20), (5, 0)))

    split_clips(video, [track], INFO, tmp_path / "split", RenderConfig(clip_margin_s=0.5, encoder=encoder))

    clips = list((tmp_path / "split").iterdir())
    assert [c.name for c in clips] == ["01_0m00s00.mp4"]
    assert duration_of(clips[0]) == pytest.approx(10 / 30 - 1 / 30 + 0.5, abs=0.1)


@requires_ffmpeg
def test_previous_clips_are_removed_before_cutting(tmp_path: Path) -> None:
    video = small_video(tmp_path / "v.mp4", frames=90)
    split = tmp_path / "split"
    split.mkdir()
    (split / "99_old.mp4").write_bytes(b"stale")

    split_clips(video, [], INFO, split, RenderConfig())

    assert list(split.iterdir()) == []


@requires_ffmpeg
def test_ffmpeg_failure_is_reported_as_a_video_error(tmp_path: Path) -> None:
    track = Track(1, line(0, 10, (20, 20), (5, 0)))

    with pytest.raises(VideoError, match="clip #1"):
        split_clips(tmp_path / "missing.mp4", [track], INFO, tmp_path / "split", RenderConfig(encoder=X264))
