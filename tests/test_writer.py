from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest

from batdetect.output.config import X264, Encoder, RenderConfig
from batdetect.output.writer import FrameWriter
from batdetect.video import VideoError, VideoInfo, open_video, read_frames
from helpers import EVERY_ENCODER, finishes, requires_ffmpeg

INFO = VideoInfo(fps=30.0, frame_count=10, width=320, height=240, work_width=320, work_height=240)
ODD = VideoInfo(fps=30.0, frame_count=10, width=321, height=240, work_width=321, work_height=240)


def gray(info: VideoInfo, level: int) -> bytes:
    return np.full((info.height, info.width, 3), level, dtype=np.uint8).tobytes()


@requires_ffmpeg
@pytest.mark.parametrize("encoder", EVERY_ENCODER)
def test_written_frames_come_back_with_their_size_and_count(tmp_path: Path, encoder: Encoder) -> None:
    out = tmp_path / "o.mp4"

    with FrameWriter(out, INFO, RenderConfig(encoder=encoder)) as writer:
        for i in range(10):
            writer.write(gray(INFO, 20 * i))

    cap, info = open_video(out, 320)
    frames = list(read_frames(cap))
    cap.release()
    assert (info.width, info.height, len(frames)) == (320, 240, 10)


@requires_ffmpeg
def test_closing_twice_is_harmless(tmp_path: Path) -> None:
    writer = FrameWriter(tmp_path / "o.mp4", INFO, RenderConfig(encoder=X264))
    writer.write(gray(INFO, 100))

    writer.close()
    writer.close()
    writer.abort()

    assert (tmp_path / "o.mp4").stat().st_size > 0


@requires_ffmpeg
def test_encoder_failure_is_a_video_error_that_leaves_no_file_and_no_ffmpeg(tmp_path: Path) -> None:
    out = tmp_path / "odd.mp4"

    writers: list[FrameWriter] = []

    raised: list[VideoError] = []

    def write_odd_frames() -> None:
        try:
            with FrameWriter(out, ODD, RenderConfig(encoder=X264)) as writer:
                writers.append(writer)
                for _ in range(30):
                    writer.write(gray(ODD, 100))
        except VideoError as error:
            raised.append(error)

    assert finishes(write_odd_frames)
    assert "odd.mp4" in str(raised[0])

    assert not out.exists()
    assert not writers[0].running


@requires_ffmpeg
def test_interruption_kills_ffmpeg_and_drops_the_partial_file(tmp_path: Path) -> None:
    out = tmp_path / "o.mp4"

    writers: list[FrameWriter] = []

    interrupted: list[bool] = []

    def interrupt_after_one_frame() -> None:
        try:
            with FrameWriter(out, INFO, RenderConfig(encoder=X264)) as writer:
                writers.append(writer)
                writer.write(gray(INFO, 100))
                raise KeyboardInterrupt
        except KeyboardInterrupt:
            interrupted.append(True)

    assert finishes(interrupt_after_one_frame)
    assert interrupted == [True]

    assert not out.exists()
    assert not writers[0].running
