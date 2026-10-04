from __future__ import annotations

from dataclasses import replace
from pathlib import Path

import cv2
import numpy as np
import pytest

from batdetect.output.clips import ClipWindow
from batdetect.output.config import X264, Encoder, RenderConfig
from batdetect.output.overlay import track_overlay
from batdetect.output.render import MAX_WRITERS, VideoOutputs, render_videos
from batdetect.track import Track
from batdetect.video import ColorFrame, GrayFrame, VideoError, VideoInfo, open_video, read_frames
from helpers import EVERY_ENCODER, line, requires_ffmpeg, small_video, write_video

FRAMES = 30
BAND = 10
DARK = 40
INFO = VideoInfo(fps=30.0, frame_count=FRAMES, width=320, height=240, work_width=320, work_height=240)


def band_frame(number: int) -> GrayFrame:
    frame = np.full((240, 320), DARK, dtype=np.uint8)
    frame[:, number * BAND : (number + 1) * BAND] = 255
    return frame


def numbered_video(path: Path) -> Path:
    write_video(path, [band_frame(i) for i in range(FRAMES)], fps=30)
    return path


def frame_numbers(path: Path) -> list[int]:
    cap, _ = open_video(path, 320)
    numbers = [int(np.argmax(frame.mean(axis=(0, 2)))) // BAND for frame in read_frames(cap)]
    cap.release()
    return numbers


def leave_as_is(_frame: ColorFrame, _frame_no: int) -> None:
    return None


def outputs(tmp_path: Path, *spans: tuple[int, int]) -> VideoOutputs:
    split = tmp_path / "split"
    clips = [ClipWindow(first, last, split / f"{first:02d}-{last:02d}.mp4") for first, last in spans]
    return VideoOutputs(split, clips, tmp_path / "boxes.mp4")


@requires_ffmpeg
@pytest.mark.parametrize("encoder", EVERY_ENCODER)
def test_each_clip_frame_is_the_source_frame_at_the_same_offset(tmp_path: Path, encoder: Encoder) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    out = outputs(tmp_path, (0, 4), (3, 9), (9, 12), (25, 40))

    render_videos(video, INFO, out, leave_as_is, RenderConfig(encoder=encoder))

    assert [frame_numbers(c.path) for c in out.clips] == [
        list(range(0, 5)),
        list(range(3, 10)),
        list(range(9, 13)),
        list(range(25, 30)),
    ]


@requires_ffmpeg
def test_more_overlapping_clips_than_writers_are_all_rendered(tmp_path: Path) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    out = outputs(tmp_path, *[(i, i + 10) for i in range(MAX_WRITERS + 2)])

    drawn: list[int] = []

    def record(_frame: ColorFrame, frame_no: int) -> None:
        drawn.append(frame_no)

    render_videos(video, INFO, out, record, RenderConfig(encoder=X264))

    assert [frame_numbers(c.path) for c in out.clips] == [list(range(i, i + 11)) for i in range(MAX_WRITERS + 2)]
    assert len(drawn) > len(set(drawn))


@requires_ffmpeg
def test_interruption_while_drawing_leaves_no_partial_clip(tmp_path: Path) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    out = outputs(tmp_path, (0, 20), (2, 25))

    def interrupt_at_ten(_frame: ColorFrame, frame_no: int) -> None:
        if frame_no == 10:
            raise KeyboardInterrupt

    with pytest.raises(KeyboardInterrupt):
        render_videos(video, INFO, out, interrupt_at_ten, RenderConfig(encoder=X264))

    assert list(out.split_dir.iterdir()) == []


@requires_ffmpeg
def test_whole_annotated_video_is_written_only_when_asked(tmp_path: Path) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    out = outputs(tmp_path, (5, 8))
    cfg = RenderConfig(encoder=X264)

    render_videos(video, INFO, out, leave_as_is, replace(cfg, annotated=True))
    assert frame_numbers(out.annotated) == list(range(FRAMES))

    render_videos(video, INFO, out, leave_as_is, cfg)
    assert not out.annotated.exists()


@requires_ffmpeg
def test_clips_show_the_drawn_box(tmp_path: Path) -> None:
    video = small_video(tmp_path / "v.mp4", frames=30)
    track = Track(1, line(5, 10, (100, 120), (4, 0)))
    out = outputs(tmp_path, (5, 14))
    cfg = RenderConfig(box_pad=10, encoder=X264)

    render_videos(video, INFO, out, track_overlay([track], INFO, cfg), cfg)

    cap, _ = open_video(out.clips[0].path, 320)
    frames = list(read_frames(cap))
    cap.release()
    hsv = cv2.cvtColor(frames[3][112:128, 100:103], cv2.COLOR_BGR2HSV)
    assert np.any((hsv[..., 1] > 150) & ((hsv[..., 0] < 10) | (hsv[..., 0] > 170)))


@requires_ffmpeg
def test_previous_clips_are_removed(tmp_path: Path) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    out = outputs(tmp_path)
    out.split_dir.mkdir()
    (out.split_dir / "99_old.mp4").write_bytes(b"stale")

    render_videos(video, INFO, out, leave_as_is, RenderConfig(encoder=X264))

    assert list(out.split_dir.iterdir()) == []


@requires_ffmpeg
def test_encoder_failure_stops_the_render_without_partial_clips(tmp_path: Path) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    width_x264_refuses = replace(INFO, width=321)
    out = outputs(tmp_path, (0, 20), (2, 25))

    with pytest.raises(VideoError, match="ffmpeg failed"):
        render_videos(video, width_x264_refuses, out, leave_as_is, RenderConfig(encoder=X264))

    assert list(out.split_dir.iterdir()) == []


def test_missing_video_raises_a_video_error(tmp_path: Path) -> None:
    with pytest.raises(VideoError, match="cannot open"):
        render_videos(tmp_path / "missing.mp4", INFO, outputs(tmp_path, (0, 3)), leave_as_is, RenderConfig())
