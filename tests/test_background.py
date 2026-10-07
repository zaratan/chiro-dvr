from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest

from batdetect.detect import DetectConfig, Region
from batdetect.output.background import hide_display, median_background
from batdetect.output.style import PANEL_COLOR
from batdetect.video import VideoError, VideoInfo, open_video
from helpers import (
    BACKGROUND,
    DECODER_ERROR,
    decoder_log,
    moving_square_frames,
    noisy_damaged_video,
    requires_ffmpeg,
    write_video,
)


def test_median_background_removes_a_moving_object(tmp_path: Path) -> None:
    video = tmp_path / "v.mp4"
    write_video(video, moving_square_frames(60, step=(2, 0)), fps=30)
    cap, info = open_video(video, 160)
    cap.release()

    image = median_background(video, info)

    assert image.shape == (120, 160, 3)
    assert float(image[60:63, 20:140].min()) > BACKGROUND - 30


def test_missing_video_raises_a_video_error(tmp_path: Path) -> None:
    with pytest.raises(VideoError):
        median_background(tmp_path / "missing.mp4", VideoInfo(30.0, 10, 320, 240, 320, 240))


def test_display_regions_are_hidden_and_the_scene_is_kept() -> None:
    scene = np.full((100, 200, 3), BACKGROUND, dtype=np.uint8)

    hidden = hide_display(scene, DetectConfig(osd_regions=(Region(0, 0, 1, 0.1),)))

    assert tuple(hidden[5, 100].tolist()) == PANEL_COLOR
    assert hidden[50, 100].tolist() == [BACKGROUND] * 3
    assert scene[5, 100].tolist() == [BACKGROUND] * 3


def test_nothing_is_hidden_without_display_regions() -> None:
    scene = np.full((100, 200, 3), BACKGROUND, dtype=np.uint8)

    assert np.array_equal(hide_display(scene, DetectConfig()), scene)


@requires_ffmpeg
def test_background_of_a_damaged_video_writes_no_decoder_message(tmp_path: Path) -> None:
    video = noisy_damaged_video(tmp_path / "cut.mp4")

    log = decoder_log(
        video, "from batdetect.output.background import median_background", "median_background(video, info)"
    )

    assert DECODER_ERROR not in log
