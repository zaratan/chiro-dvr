from __future__ import annotations

from pathlib import Path

import pytest

from batdetect.jobs import collect_videos, plan_jobs


def test_collect_videos_keeps_only_video_files_of_a_folder(tmp_path: Path) -> None:
    for name in ["b.MP4", "a.mov", "notes.txt", "c.mkv"]:
        (tmp_path / name).touch()
    (tmp_path / "sub.mp4").mkdir()

    assert [p.name for p in collect_videos([tmp_path])] == ["a.mov", "b.MP4", "c.mkv"]


def test_videos_with_the_same_name_get_distinct_output_folders(tmp_path: Path) -> None:
    videos = [Path("night1/video_001.mp4"), Path("night2/video_001.mp4"), Path("night2/video_002.mp4")]

    dests = [job.dest for job in plan_jobs(videos, tmp_path)]

    assert dests == [tmp_path / "night1_video_001_mp4", tmp_path / "night2_video_001_mp4", tmp_path / "video_002"]


def test_same_name_with_another_extension_in_the_same_folder_gets_its_own_folder(tmp_path: Path) -> None:
    dests = [job.dest for job in plan_jobs([Path("night1/x.mp4"), Path("night1/x.mov")], tmp_path)]

    assert dests == [tmp_path / "night1_x_mp4", tmp_path / "night1_x_mov"]


def test_videos_that_would_share_an_output_folder_are_refused(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="several videos"):
        plan_jobs([Path("a/night/x.mp4"), Path("b/night/x.mp4")], tmp_path)
