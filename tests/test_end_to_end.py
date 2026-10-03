from __future__ import annotations

import csv
import json
import shutil
from pathlib import Path

import pytest

from batdetect.cli import main
from batdetect.pipeline import DetectConfig, TrackConfig, detect_frames, open_video, read_gray_frames, track_detections
from helpers import moving_square_frames, write_video

FIXTURE = Path(__file__).parent / "fixtures" / "video_092_3m24-4m05.mp4"
FIXTURE_START_S = 6148 / 30

requires_ffmpeg = pytest.mark.skipif(shutil.which("ffmpeg") is None, reason="ffmpeg not installed")


@requires_ffmpeg
def test_folder_run_writes_every_output_for_each_video(tmp_path: Path) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    write_video(videos / "flight.mp4", moving_square_frames(90, width=320, height=240, step=(3, 1)), fps=30)
    out = tmp_path / "out"

    assert main([str(videos), "-o", str(out)]) == 0

    dest = out / "flight"
    with (dest / "flight.tracks.csv").open() as fh:
        rows = list(csv.DictReader(fh))
    assert len(rows) == 1
    assert (dest / "flight.tracks.png").stat().st_size > 0
    assert (dest / "flight_boxes.mp4").stat().st_size > 0
    assert [p.name for p in (dest / "split").iterdir()] == ["01_0m00s00.mp4"]
    assert json.loads((dest / "params.json").read_text())["TrackConfig"]["min_hits"] == 5


@requires_ffmpeg
def test_unreadable_video_fails_without_stopping_the_others(tmp_path: Path) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    (videos / "broken.mp4").write_bytes(b"not a video")
    write_video(videos / "ok.mp4", moving_square_frames(60, width=320, height=240), fps=30)

    assert main([str(videos), "-o", str(tmp_path / "out")]) == 1
    assert (tmp_path / "out" / "ok" / "ok.tracks.csv").exists()


@pytest.mark.slow
def test_bats_confirmed_by_the_naturalist_are_tracked() -> None:
    cap, info = open_video(FIXTURE, DetectConfig().work_width)
    try:
        detections = detect_frames(read_gray_frames(cap, info), info.fps, DetectConfig())
    finally:
        cap.release()
    tracks = track_detections(detections, TrackConfig())

    def active_at(source_s: float) -> bool:
        frame = (source_s - FIXTURE_START_S) * info.fps
        return any(t.first.frame - info.fps <= frame <= t.last.frame + info.fps for t in tracks)

    assert active_at(3 * 60 + 36)
    assert active_at(3 * 60 + 58)
    assert 4 <= len(tracks) <= 6
