from __future__ import annotations

import unicodedata
from pathlib import Path

import pytest

from batdetect.output.names import (
    annotated_video,
    clip_name,
    clips_dir,
    params_json,
    previous_outputs,
    remove_previous_outputs,
    summary_png,
    tracks_csv,
    zoom_path,
)
from batdetect.track import Track
from helpers import line

DEST = Path("out/v")


def starting_at(track_id: int, frame: int) -> Track:
    return Track(track_id, line(frame, 5, (0, 0), (1, 0)))


def every_name() -> list[str]:
    clip = clips_dir(DEST) / clip_name(starting_at(1, 0), 30.0, 1)
    paths = [clip, zoom_path(clip), tracks_csv(DEST, "v"), summary_png(DEST, "v"), annotated_video(DEST, "v")]
    return [str(p) for p in paths]


def test_english_names_when_the_locale_is_not_french() -> None:
    assert every_name() == [
        "out/v/clips/track_01_0m00s.mp4",
        "out/v/clips/track_01_0m00s_zoom.mp4",
        "out/v/v_tracks.csv",
        "out/v/v_summary.png",
        "out/v/v_annotated.mp4",
    ]


@pytest.mark.usefixtures("french")
def test_french_names_under_a_french_locale_without_any_accent_that_apfs_would_decompose() -> None:
    names = every_name()

    assert names == [
        "out/v/extraits/piste_01_0m00s.mp4",
        "out/v/extraits/piste_01_0m00s_zoom.mp4",
        "out/v/v_pistes.csv",
        "out/v/v_resume.png",
        "out/v/v_annotee.mp4",
    ]
    assert all(name.isascii() for name in names)


@pytest.mark.usefixtures("french")
def test_params_json_keeps_its_english_name_under_a_french_locale() -> None:
    assert params_json(DEST) == DEST / "params.json"


def test_clip_time_drops_hundredths_and_the_track_number_tells_apart_two_tracks_of_the_same_second() -> None:
    names = [clip_name(starting_at(7, 7122), 30.0, 8), clip_name(starting_at(8, 7139), 30.0, 8)]

    assert names == ["track_07_3m57s.mp4", "track_08_3m57s.mp4"]


def test_clip_time_counts_minutes_past_the_hour() -> None:
    assert clip_name(starting_at(12, 30 * 3725), 30.0, 12) == "track_12_62m05s.mp4"


@pytest.mark.parametrize(
    ("track_count", "expected"),
    [(99, "track_07_0m00s.mp4"), (100, "track_007_0m00s.mp4"), (1000, "track_0007_0m00s.mp4")],
)
def test_track_number_takes_the_width_of_the_track_count_so_the_finder_sorts_clips_in_numeric_order(
    track_count: int, expected: str
) -> None:
    assert clip_name(starting_at(7, 0), 30.0, track_count) == expected


@pytest.mark.usefixtures("french")
def test_french_and_zoomed_clips_follow_the_same_number_width() -> None:
    clip = clips_dir(DEST) / clip_name(starting_at(7, 0), 30.0, 100)

    assert [clip.name, zoom_path(clip).name] == ["piste_007_0m00s.mp4", "piste_007_0m00s_zoom.mp4"]


def test_summary_of_a_long_video_carries_its_period() -> None:
    assert summary_png(DEST, "v", "_000m-010m") == DEST / "v_summary_000m-010m.png"


def test_cleanup_removes_outputs_of_both_languages_and_of_the_old_names_but_nothing_else(tmp_path: Path) -> None:
    removed = [
        "split",
        "clips",
        "extraits",
        "v.tracks.csv",
        "v_tracks.csv",
        "v_pistes.csv",
        "v.tracks.png",
        "v.tracks_000m-010m.png",
        "v_summary.png",
        "v_resume_010m-020m.png",
        "v_boxes.mp4",
        "v_annotated.mp4",
        "v_annotee.mp4",
    ]
    kept = ["params.json", "notes.txt", "w_tracks.csv", "v_summary.txt", "v_resume_annote.png", "v_summary_old.png"]
    for name in removed + kept:
        path = tmp_path / name
        if "." in name:
            path.write_text("old")
        else:
            path.mkdir()
            (path / "01_0m04s80.mp4").write_text("old")

    assert sorted(p.name for p in previous_outputs(tmp_path, "v")) == sorted(removed)
    remove_previous_outputs(tmp_path, "v")

    assert sorted(p.name for p in tmp_path.iterdir()) == sorted(kept)


def test_cleanup_of_a_folder_not_yet_created_finds_nothing(tmp_path: Path) -> None:
    assert previous_outputs(tmp_path / "missing", "v") == []


def test_cleanup_finds_outputs_whatever_the_unicode_form_of_an_accented_video_name(tmp_path: Path) -> None:
    decomposed = unicodedata.normalize("NFD", "été")
    (tmp_path / f"{decomposed}_tracks.csv").write_text("old")
    (tmp_path / f"{decomposed}.tracks.png").write_text("old")

    remove_previous_outputs(tmp_path, unicodedata.normalize("NFC", "été"))

    assert list(tmp_path.iterdir()) == []
