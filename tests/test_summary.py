from __future__ import annotations

from pathlib import Path

import cv2
import numpy as np
import pytest

from batdetect.output.summary import summary_image
from batdetect.track import Track
from batdetect.video import VideoError, VideoInfo, open_video
from helpers import BACKGROUND, line, small_video


def test_summary_darkens_the_background_and_draws_each_track(tmp_path: Path) -> None:
    video = small_video(tmp_path / "v.mp4", frames=40)
    cap, info = open_video(video, 320)
    cap.release()
    out = tmp_path / "summary.png"

    summary_image(video, [Track(1, line(0, 20, (40, 200), (10, -5)))], info, out)

    image = np.asarray(cv2.imread(str(out)))
    assert image.shape == (240, 320, 3)
    assert image[10, 300].mean() == pytest.approx(BACKGROUND * 0.6, abs=10)
    on_trail = [int(v) for v in image[197, 45]]
    assert max(on_trail) - min(on_trail) > 100


def test_unreadable_video_raises_a_video_error(tmp_path: Path) -> None:
    info = VideoInfo(30.0, 10, 320, 240, 320, 240)

    with pytest.raises(VideoError):
        summary_image(tmp_path / "missing.mp4", [], info, tmp_path / "s.png")
