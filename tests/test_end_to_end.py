from __future__ import annotations

import csv
import json
from pathlib import Path

import numpy as np
import pytest

from batdetect.cli import main
from batdetect.track import Track
from batdetect.video import GrayFrame
from helpers import (
    NOISE,
    assert_matches_reference,
    background,
    requires_ffmpeg,
    tracks_with_defaults,
    with_square,
    write_video,
)

FIXTURE = Path(__file__).parent / "fixtures" / "video_092_original_3m24-4m05.mp4"
SLIDE_STEP = 4
FIXTURE_START_S = 6150 * 333 / 10000


def sliding_scene_then_flight(frames: int, slide: range, flight: range) -> list[GrayFrame]:
    rng = np.random.default_rng(3)
    canvas = background(360, 320, seed=0).astype(np.int16)
    for x, y in zip(rng.integers(5, 350, 200), rng.integers(5, 310, 200), strict=True):
        canvas[y : y + 3, x : x + 3] -= 90
    out: list[GrayFrame] = []
    for i in range(frames):
        offset = SLIDE_STEP * min(max(i - slide.start, 0), len(slide))
        noise = rng.normal(0, NOISE, (240, 320))
        frame = np.clip(canvas[10 + offset : 250 + offset, 20:340] + noise, 0, 255).astype(np.uint8)
        if i in flight:
            frame = with_square(frame, 20 + 6 * (i - flight.start), 120, size=6)
        out.append(frame)
    return out


def flying_square(count: int, step: tuple[int, int]) -> list[GrayFrame]:
    return [with_square(background(320, 240, seed=i), 20 + step[0] * i, 60 + step[1] * i, size=6) for i in range(count)]


@requires_ffmpeg
def test_folder_run_writes_every_output_for_each_video(tmp_path: Path) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    write_video(videos / "flight.mp4", flying_square(90, step=(3, 1)), fps=30)
    out = tmp_path / "out"
    dest = out / "flight"
    dest.mkdir(parents=True)
    (dest / "flight_boxes.mp4").write_bytes(b"from a previous run")

    assert main([str(videos), "-o", str(out)]) == 0

    with (dest / "flight.tracks.csv").open() as fh:
        rows = list(csv.DictReader(fh))
    assert len(rows) == 1
    assert "filled_frames" in rows[0]
    assert (dest / "flight.tracks.png").stat().st_size > 0
    assert not (dest / "flight_boxes.mp4").exists()
    assert [p.name for p in (dest / "split").iterdir()] == ["01_0m00s00.mp4"]
    params = json.loads((dest / "params.json").read_text())
    assert params["TrackConfig"]["min_hits"] == 6
    assert params["RenderConfig"]["encoder"] in {"x264", "videotoolbox"}


@requires_ffmpeg
def test_annotated_option_adds_the_whole_annotated_video(tmp_path: Path) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    write_video(videos / "flight.mp4", flying_square(90, step=(3, 1)), fps=30)

    assert main([str(videos), "-o", str(tmp_path / "out"), "--annotated"]) == 0

    assert (tmp_path / "out" / "flight" / "flight_boxes.mp4").stat().st_size > 0


@requires_ffmpeg
def test_frames_where_the_camera_slides_are_ignored_and_reported(tmp_path: Path) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    write_video(videos / "shaky.mp4", sliding_scene_then_flight(150, range(30, 45), range(100, 140)), fps=30)

    assert main([str(videos), "-o", str(tmp_path / "raw"), "--max-blobs", "0"]) == 0
    assert main([str(videos), "-o", str(tmp_path / "out")]) == 0

    def starts(root: Path) -> list[float]:
        with (root / "shaky" / "shaky.tracks.csv").open() as fh:
            return [float(row["start_s"]) for row in csv.DictReader(fh)]

    assert sum(1 for s in starts(tmp_path / "raw") if 0.5 < s < 2.0) >= 5
    assert [round(s, 1) for s in starts(tmp_path / "out")] == [3.3]
    ignored = json.loads((tmp_path / "out" / "shaky" / "params.json").read_text())["ignored_s"]
    assert ignored == [pytest.approx([0.0, 2.5], abs=0.04)]


@requires_ffmpeg
def test_unreadable_video_fails_without_stopping_the_others(tmp_path: Path) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    (videos / "broken.mp4").write_bytes(b"not a video")
    write_video(videos / "ok.mp4", flying_square(60, step=(3, 0)), fps=30)

    assert main([str(videos), "-o", str(tmp_path / "out")]) == 1
    assert (tmp_path / "out" / "ok" / "ok.tracks.csv").exists()


@pytest.fixture(scope="module")
def extract_tracks() -> tuple[list[Track], float]:
    return tracks_with_defaults(FIXTURE)


@pytest.mark.slow
def test_bats_confirmed_by_the_naturalist_are_tracked(extract_tracks: tuple[list[Track], float]) -> None:
    tracks, fps = extract_tracks

    def active_at(source_s: float) -> bool:
        frame = (source_s - FIXTURE_START_S) * fps
        return any(t.first.frame - fps <= frame <= t.last.frame + fps for t in tracks)

    assert active_at(3 * 60 + 36)
    assert active_at(3 * 60 + 58)


@pytest.mark.slow
def test_extract_keeps_its_four_tracks_within_the_noise_between_platforms(
    extract_tracks: tuple[list[Track], float],
) -> None:
    assert_matches_reference(extract_tracks[0], start_frames=[144, 705, 787, 976], hits=[287, 16, 21, 18])
