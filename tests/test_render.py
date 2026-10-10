from __future__ import annotations

import subprocess
from dataclasses import replace
from fractions import Fraction
from pathlib import Path

import cv2
import numpy as np
import pytest

from batdetect.output.clips import ClipWindow
from batdetect.output.config import X264, Encoder, RenderConfig
from batdetect.output.overlay import track_overlay
from batdetect.output.render import MAX_WRITERS, VideoOutputs, render_videos
from batdetect.output.zoom import SLOW_MOTION, Crop
from batdetect.output.zoomview import zoom_view, zoom_windows
from batdetect.track import Track
from batdetect.video import ColorFrame, GrayFrame, VideoError, VideoInfo, open_video, read_frames
from helpers import (
    DECODER_ERROR,
    EVERY_ENCODER,
    decoder_log,
    line,
    noisy_damaged_video,
    requires_ffmpeg,
    small_video,
    write_video,
)

FRAMES = 30
BAND = 10
DARK = 40
BINOCULARS_FPS = 30.02738820371172
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
    folder = tmp_path / "clips"
    clips = [ClipWindow(first, last, folder / f"{first:02d}-{last:02d}.mp4") for first, last in spans]
    return VideoOutputs(folder, clips, tmp_path / "annotated.mp4")


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

    assert list(out.clips_dir.iterdir()) == []


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
    out.clips_dir.mkdir()
    (out.clips_dir / "99_old.mp4").write_bytes(b"stale")

    render_videos(video, INFO, out, leave_as_is, RenderConfig(encoder=X264))

    assert list(out.clips_dir.iterdir()) == []


@requires_ffmpeg
def test_encoder_failure_stops_the_render_without_partial_clips(tmp_path: Path) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    width_x264_refuses = replace(INFO, width=321)
    out = outputs(tmp_path, (0, 20), (2, 25))

    with pytest.raises(VideoError, match="ffmpeg failed"):
        render_videos(video, width_x264_refuses, out, leave_as_is, RenderConfig(encoder=X264))

    assert list(out.clips_dir.iterdir()) == []


def test_missing_video_raises_a_video_error(tmp_path: Path) -> None:
    with pytest.raises(VideoError, match="cannot open"):
        render_videos(tmp_path / "missing.mp4", INFO, outputs(tmp_path, (0, 3)), leave_as_is, RenderConfig())


@requires_ffmpeg
def test_rendering_a_damaged_video_writes_no_decoder_message(tmp_path: Path) -> None:
    video = noisy_damaged_video(tmp_path / "cut.mp4")
    clip = tmp_path / "out" / "clip.mp4"

    log = decoder_log(
        video,
        "from batdetect.output.clips import ClipWindow",
        "from batdetect.output.config import X264, RenderConfig",
        "from batdetect.output.render import VideoOutputs, render_videos",
        f"clips = [ClipWindow(0, 119, Path({str(clip)!r}))]",
        f"outputs = VideoOutputs(Path({str(clip.parent)!r}), clips, Path({str(tmp_path / 'whole.mp4')!r}))",
        "render_videos(video, info, outputs, lambda frame, frame_no: None, RenderConfig(encoder=X264))",
    )

    assert clip.exists()
    assert DECODER_ERROR not in log


def declared_rate(path: Path) -> Fraction:
    probe = subprocess.run(
        [
            "ffprobe",
            "-v",
            "error",
            "-select_streams",
            "v",
            "-show_entries",
            "stream=r_frame_rate",
            "-of",
            "csv=p=0",
            path,
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    return Fraction(probe.stdout.strip())


def saturated_pixels(path: Path, hue_low: int, hue_high: int) -> int:
    cap, _ = open_video(path, 320)
    frames = list(read_frames(cap))
    cap.release()
    count = 0
    for frame in frames:
        hsv = cv2.cvtColor(frame, cv2.COLOR_BGR2HSV)
        count += int(
            np.sum((hsv[..., 1] > 150) & (hsv[..., 2] > 100) & (hsv[..., 0] >= hue_low) & (hsv[..., 0] <= hue_high))
        )
    return count


def red_pixels(path: Path) -> int:
    return saturated_pixels(path, 0, 9) + saturated_pixels(path, 171, 180)


def trail_pixels(path: Path) -> int:
    return saturated_pixels(path, 15, 35)


def zoomed(window: ClipWindow, track: Track, crop: Crop, cfg: RenderConfig) -> ClipWindow:
    path = window.path.with_stem(window.path.stem + "_zoom")
    return ClipWindow(window.first, window.last, path, zoom_view(track, crop, INFO, cfg), SLOW_MOTION)


@requires_ffmpeg
@pytest.mark.parametrize("encoder", EVERY_ENCODER)
def test_zoomed_clip_has_the_same_frames_declared_at_a_quarter_of_the_rate(tmp_path: Path, encoder: Encoder) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    cfg = RenderConfig(encoder=encoder)
    out = outputs(tmp_path, (3, 12))
    far_away = Track(1, line(50, 6, (0, 0), (1, 0)))
    out = replace(out, clips=[*out.clips, zoomed(out.clips[0], far_away, Crop(0, 0, 320, 240), cfg)])

    render_videos(video, replace(INFO, fps=BINOCULARS_FPS), out, leave_as_is, cfg)

    normal, zoom = (c.path for c in out.clips)
    assert frame_numbers(zoom) == frame_numbers(normal) == list(range(3, 13))
    assert float(declared_rate(normal) / declared_rate(zoom)) == pytest.approx(SLOW_MOTION, rel=1e-6)


@requires_ffmpeg
@pytest.mark.parametrize("crop", [Crop(80, 60, 80, 60), Crop(0, 0, 320, 240)])
def test_normal_clip_bytes_do_not_change_when_a_zoom_runs_alongside(tmp_path: Path, crop: Crop) -> None:
    video = small_video(tmp_path / "v.mp4", frames=30)
    track = Track(1, line(5, 10, (100, 80), (3, 1)))
    cfg = RenderConfig(encoder=X264)
    alone = outputs(tmp_path, (2, 20))
    render_videos(video, INFO, alone, leave_as_is, cfg)
    reference = alone.clips[0].path.read_bytes()

    with_zoom = replace(alone, clips=[*alone.clips, zoomed(alone.clips[0], track, crop, cfg)])
    render_videos(video, INFO, with_zoom, leave_as_is, cfg)

    assert with_zoom.clips[0].path.read_bytes() == reference
    assert with_zoom.clips[1].path.exists()


@requires_ffmpeg
def test_zoomed_clip_shows_the_trail_but_not_the_box_of_the_normal_clip(tmp_path: Path) -> None:
    video = small_video(tmp_path / "v.mp4", frames=30)
    track = Track(1, line(5, 15, (90, 90), (3, 1)))
    cfg = RenderConfig(box_pad=6, encoder=X264)
    out = outputs(tmp_path, (2, 25))
    out = replace(out, clips=[*out.clips, zoomed(out.clips[0], track, Crop(60, 60, 80, 60), cfg)])

    render_videos(video, INFO, out, track_overlay([track], INFO, cfg), cfg)

    normal, zoom = (c.path for c in out.clips)
    assert red_pixels(normal) > 0
    assert red_pixels(zoom) == 0
    assert trail_pixels(zoom) > 0


@requires_ffmpeg
def test_zooms_beyond_the_writer_limit_are_rendered_in_further_passes(tmp_path: Path) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    cfg = RenderConfig(encoder=X264, zoom="all")
    out = outputs(tmp_path, *[(i, i + 10) for i in range(MAX_WRITERS)])
    tracks = [Track(i, line(i + 2, 6, (100, 100), (5, 0))) for i in range(MAX_WRITERS)]
    out = replace(out, clips=[*out.clips, *zoom_windows(tracks, out.clips, INFO, cfg)])

    render_videos(video, INFO, out, leave_as_is, cfg)

    assert len(out.clips) == 2 * MAX_WRITERS
    assert [len(frame_numbers(c.path)) for c in out.clips] == [11] * (2 * MAX_WRITERS)


@requires_ffmpeg
def test_interruption_leaves_no_partial_zoomed_clip(tmp_path: Path) -> None:
    video = numbered_video(tmp_path / "v.mp4")
    cfg = RenderConfig(encoder=X264, zoom="all")
    out = outputs(tmp_path, (0, 20))
    track = Track(1, line(2, 8, (100, 100), (5, 0)))
    out = replace(out, clips=[*out.clips, *zoom_windows([track], out.clips, INFO, cfg)])

    def interrupt_at_ten(_frame: ColorFrame, frame_no: int) -> None:
        if frame_no == 10:
            raise KeyboardInterrupt

    with pytest.raises(KeyboardInterrupt):
        render_videos(video, INFO, out, interrupt_at_ten, cfg)

    assert list(out.clips_dir.iterdir()) == []
