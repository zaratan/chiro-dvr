from __future__ import annotations

import csv
import json
from collections.abc import Generator
from contextlib import contextmanager
from dataclasses import replace
from pathlib import Path

import numpy as np
import pytest

from batdetect.cli import main
from batdetect.probe import ProbeOutcome, probing
from batdetect.track import Track
from batdetect.video import GrayFrame
from helpers import (
    NOISE,
    assert_matches_reference,
    background,
    damaged_video,
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


def two_squares_flying(count: int) -> list[GrayFrame]:
    return [
        with_square(with_square(background(320, 240, seed=i), 20 + 3 * i, 40, size=6), 20 + 3 * i, 180, size=6)
        for i in range(count)
    ]


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
    assert sorted(p.name for p in (dest / "split").iterdir()) == ["01_0m00s00.mp4", "01_0m00s00_zoom.mp4"]
    params = json.loads((dest / "params.json").read_text())
    assert params["TrackConfig"]["min_hits"] == 6
    assert params["mode"] == "normal"
    assert params["RenderConfig"]["encoder"] in {"x264", "videotoolbox"}
    assert (params["damaged_s"], params["damaged_frames"]) == ([], 0)
    assert params["ffprobe"].startswith("ffprobe version")


@requires_ffmpeg
def test_zoom_none_writes_only_the_normal_clip_of_a_small_target(tmp_path: Path) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    write_video(videos / "flight.mp4", flying_square(90, step=(3, 1)), fps=30)

    assert main([str(videos), "-o", str(tmp_path / "out"), "--zoom", "none"]) == 0

    split = tmp_path / "out" / "flight" / "split"
    assert [p.name for p in split.iterdir()] == ["01_0m00s00.mp4"]


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


@requires_ffmpeg
def test_damaged_time_is_reported_apart_and_counted_in_the_ignored_total(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    damaged_video(videos / "cut.mp4", {20, 21, 50})

    assert main([str(videos), "-o", str(tmp_path / "out")]) == 0

    params = json.loads((tmp_path / "out" / "cut" / "params.json").read_text())
    assert params["damaged_s"] == [[0.17, 2.5]]
    assert params["damaged_frames"] == 3
    assert params["ignored_s"] == []
    out = capsys.readouterr().out
    assert "2.3 s ignored" in out
    assert "  damaged 2.3 s in 1 span, detections ignored" in out


@requires_ffmpeg
def test_video_whose_frame_numbers_cannot_be_matched_fails_without_stopping_the_others(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    write_video(videos / "lost.mp4", flying_square(60, step=(3, 0)), fps=30)
    write_video(videos / "ok.mp4", flying_square(60, step=(3, 0)), fps=30)

    @contextmanager
    def probing_one_frame_more_on_lost(video: Path) -> Generator[ProbeOutcome]:
        with probing(video) as outcome:
            yield outcome
        if video.name == "lost.mp4":
            outcome.found = replace(outcome.probe, frame_count=outcome.probe.frame_count + 1)

    monkeypatch.setattr("batdetect.cli.probing", probing_one_frame_more_on_lost)

    assert main([str(videos), "-o", str(tmp_path / "out")]) == 1
    assert "frame numbers do not match" in capsys.readouterr().err
    assert (tmp_path / "out" / "ok" / "ok.tracks.csv").exists()
    assert not (tmp_path / "out" / "lost").exists()


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
    assert_matches_reference(extract_tracks[0], start_frames=[144, 705, 783, 972], hits=[290, 17, 24, 27])


@requires_ffmpeg
def test_video_above_max_tracks_keeps_its_tables_and_summary_skips_its_clips_and_fails_without_stopping_the_batch(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    write_video(videos / "crowded.mp4", two_squares_flying(90), fps=30)
    write_video(videos / "flight.mp4", flying_square(90, step=(3, 1)), fps=30)
    out = tmp_path / "out"
    crowded = out / "crowded"
    (crowded / "split").mkdir(parents=True)
    (crowded / "crowded_boxes.mp4").write_bytes(b"from a previous run")

    code = main([str(videos), "-o", str(out), "--max-tracks", "1", "--annotated", "--zoom", "none"])

    with (crowded / "crowded.tracks.csv").open() as fh:
        assert len(list(csv.DictReader(fh))) == 2
    assert (crowded / "crowded.tracks.png").stat().st_size > 0
    assert json.loads((crowded / "params.json").read_text())["RenderConfig"]["max_tracks"] == 1
    assert not (crowded / "split").exists()
    assert not (crowded / "crowded_boxes.mp4").exists()
    assert len(list((out / "flight" / "split").iterdir())) == 1
    assert (out / "flight" / "flight_boxes.mp4").exists()
    assert code == 1
    assert "crowded.mp4: 2 tracks, above --max-tracks 1" in capsys.readouterr().err


@requires_ffmpeg
def test_video_above_max_tracks_prints_its_summary_line_but_not_its_track_list_which_the_csv_already_holds(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    write_video(videos / "crowded.mp4", two_squares_flying(90), fps=30)

    main([str(videos), "-o", str(tmp_path / "out"), "--max-tracks", "1"])

    printed = capsys.readouterr().out
    assert "crowded.mp4: 90 frames, 2 tracks" in printed
    assert "#1" not in printed


@requires_ffmpeg
@pytest.mark.usefixtures("french")
def test_end_of_run_lines_are_in_french_with_a_decimal_comma_under_a_french_locale(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    damaged_video(videos / "cut.mp4", {20, 21, 50})
    write_video(videos / "flight.mp4", flying_square(90, step=(3, 1)), fps=30)

    assert main([str(videos), "-o", str(tmp_path / "out")]) == 0

    out = capsys.readouterr().out
    assert "2,3 s ignorées" in out
    assert "  2,3 s abîmées sur 1 plage, détections ignorées" in out
    assert "flight.mp4 : 90 images, 1 piste, 0,0 s ignorées" in out
    assert "  #1   0:00.00 -> " in out
    assert "points=" in out


@requires_ffmpeg
@pytest.mark.usefixtures("french")
def test_too_many_tracks_is_reported_in_french_under_a_french_locale(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    videos = tmp_path / "in"
    videos.mkdir()
    write_video(videos / "crowded.mp4", two_squares_flying(90), fps=30)

    assert main([str(videos), "-o", str(tmp_path / "out"), "--max-tracks", "1"]) == 1

    assert "2 pistes, au-delà de --max-tracks 1 : extraits et vidéo annotée non écrits" in capsys.readouterr().err
