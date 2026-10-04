from __future__ import annotations

import argparse
from collections.abc import Callable
from pathlib import Path

import pytest

from batdetect.cli import collect_videos, parse_region, plan_jobs
from batdetect.output import RenderConfig, clip_name, filled_points, format_time
from batdetect.pipeline import Track
from helpers import detection, line


@pytest.mark.parametrize(
    ("seconds", "expected"),
    [(0, "0:00.00"), (7.53, "0:07.53"), (59.996, "1:00.00"), (119.997, "2:00.00"), (237.4, "3:57.40")],
)
def test_format_time_never_shows_sixty_seconds(seconds: float, expected: str) -> None:
    assert format_time(seconds) == expected


def test_clip_name_sorts_by_id_and_carries_start_time() -> None:
    track = Track(7, line(7122, 5, (0, 0), (1, 0)))

    assert clip_name(track, 30.0) == "07_3m57s40.mp4"


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


@pytest.mark.parametrize(
    "build",
    [
        lambda: RenderConfig(box_pad=-1),
        lambda: RenderConfig(trail_s=-1),
        lambda: RenderConfig(clip_margin_s=-1),
        lambda: RenderConfig(crf=52),
    ],
)
def test_invalid_render_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match="must be"):
        build()


def test_osd_region_is_parsed_from_four_fractions() -> None:
    region = parse_region("0,0.9,0.5,1")

    assert (region.x0, region.y0, region.x1, region.y1) == (0, 0.9, 0.5, 1)


@pytest.mark.parametrize("text", ["0,0,1", "a,b,c,d", "0.5,0,0.2,1"])
def test_malformed_osd_region_is_refused(text: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError):
        parse_region(text)


def test_gaps_are_filled_by_linear_interpolation_marked_as_such() -> None:
    track = Track(1, [detection(10, 0, 0), detection(13, 30, 60)])

    filled = filled_points(track)

    assert [(d.frame, round(d.x), round(d.y), interpolated) for d, interpolated in filled] == [
        (10, 0, 0, False),
        (11, 10, 20, True),
        (12, 20, 40, True),
        (13, 30, 60, False),
    ]


def test_filling_gaps_leaves_the_detections_untouched() -> None:
    track = Track(1, [detection(10, 0, 0), detection(13, 30, 60)])

    filled_points(track)

    assert [d.frame for d in track.points] == [10, 13]
