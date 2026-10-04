from __future__ import annotations

from pathlib import Path

import cv2
import numpy as np
import pytest

from batdetect.output.annotated import render_annotated
from batdetect.output.config import X264, RenderConfig
from batdetect.track import Track
from batdetect.video import VideoError, open_video, read_frames
from helpers import EVERY_ENCODER, line, requires_ffmpeg, small_video


@requires_ffmpeg
@pytest.mark.parametrize("encoder", EVERY_ENCODER)
def test_annotated_video_keeps_size_and_frame_count_and_shows_the_box(tmp_path: Path, encoder: str) -> None:
    video = small_video(tmp_path / "v.mp4", frames=30)
    cap, info = open_video(video, 320)
    cap.release()
    track = Track(1, line(5, 10, (100, 120), (4, 0)))
    out = tmp_path / "boxes.mp4"

    render_annotated(video, [track], info, out, RenderConfig(box_pad=10, encoder=encoder))

    cap, rendered = open_video(out, 320)
    frames = list(read_frames(cap))
    cap.release()
    assert (rendered.width, rendered.height, len(frames)) == (320, 240, 30)
    left_edge = frames[8][112:128, 100:103]
    hsv = cv2.cvtColor(left_edge, cv2.COLOR_BGR2HSV)
    assert np.any((hsv[..., 1] > 150) & ((hsv[..., 0] < 10) | (hsv[..., 0] > 170)))


@requires_ffmpeg
def test_missing_video_raises_a_video_error(tmp_path: Path) -> None:
    video = small_video(tmp_path / "v.mp4", frames=5)
    cap, info = open_video(video, 320)
    cap.release()

    with pytest.raises(VideoError, match="cannot open"):
        render_annotated(tmp_path / "missing.mp4", [], info, tmp_path / "o.mp4", RenderConfig(encoder=X264))
