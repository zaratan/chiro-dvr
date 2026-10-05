from __future__ import annotations

import shutil
import threading
from collections.abc import Callable, Iterable
from pathlib import Path

import cv2
import numpy as np
import pytest

from batdetect.detect import DetectConfig, Detection
from batdetect.output.config import VIDEOTOOLBOX, X264, RenderConfig
from batdetect.output.encoder import encoder_failure
from batdetect.parallel import detect_video
from batdetect.stability import StabilityConfig, unstable_spans, without_spans
from batdetect.track import Track, TrackConfig, track_detections
from batdetect.video import GrayFrame, open_video

BACKGROUND = 200
GUARD_TIMEOUT_S = 5.0
NOISE = 3
START_TOLERANCE_FRAMES = 5
HITS_TOLERANCE = 3


def detection(frame: int, x: float, y: float, area: int = 9) -> Detection:
    return Detection(frame, x, y, int(x) - 1, int(y) - 1, 3, 3, area, 80.0)


def line(start_frame: int, frames: int, start: tuple[float, float], step: tuple[float, float]) -> list[Detection]:
    return [detection(start_frame + i, start[0] + step[0] * i, start[1] + step[1] * i) for i in range(frames)]


def by_frame(*tracks: Iterable[Detection]) -> dict[int, list[Detection]]:
    out: dict[int, list[Detection]] = {}
    for det in (d for t in tracks for d in t):
        out.setdefault(det.frame, []).append(det)
    return out


def contiguous(detections: Iterable[Detection], frames: int) -> dict[int, list[Detection]]:
    out: dict[int, list[Detection]] = {f: [] for f in range(frames)}
    for det in detections:
        out[det.frame].append(det)
    return out


def tracks_with_defaults(video: Path) -> tuple[list[Track], float]:
    detect = DetectConfig()
    cap, info = open_video(video, detect.work_width)
    cap.release()
    detections = detect_video(video, detect, info, workers=1)[0]
    spans = unstable_spans(detections, info.fps, StabilityConfig())
    return track_detections(without_spans(detections, spans), TrackConfig()), info.fps


def assert_matches_reference(tracks: list[Track], start_frames: list[int], hits: list[int]) -> None:
    assert [t.first.frame for t in tracks] == pytest.approx(start_frames, abs=START_TOLERANCE_FRAMES)
    assert [len(t.points) for t in tracks] == pytest.approx(hits, abs=HITS_TOLERANCE)


def background(width: int, height: int, seed: int = 0) -> GrayFrame:
    rng = np.random.default_rng(seed)
    base = rng.normal(BACKGROUND, NOISE, (height, width))
    return np.clip(base, 0, 255).astype(np.uint8)


def with_square(frame: GrayFrame, x: int, y: int, size: int = 3, delta: int = -80) -> GrayFrame:
    out = frame.astype(np.int16)
    out[y : y + size, x : x + size] += delta
    return np.clip(out, 0, 255).astype(np.uint8)


def moving_square_frames(
    count: int, width: int = 160, height: int = 120, start: tuple[int, int] = (20, 60), step: tuple[int, int] = (3, 0)
) -> list[GrayFrame]:
    frames: list[GrayFrame] = []
    for i in range(count):
        noisy = background(width, height, seed=i)
        frames.append(with_square(noisy, start[0] + step[0] * i, start[1] + step[1] * i))
    return frames


def write_video(path: Path, frames: list[GrayFrame], fps: float) -> None:
    height, width = frames[0].shape
    writer = cv2.VideoWriter(str(path), cv2.VideoWriter.fourcc(*"mp4v"), fps, (width, height))
    for gray in frames:
        writer.write(cv2.cvtColor(gray, cv2.COLOR_GRAY2BGR))
    writer.release()


def finishes(action: Callable[[], None]) -> bool:
    guard = threading.Thread(target=action, daemon=True)
    guard.start()
    guard.join(GUARD_TIMEOUT_S)
    return not guard.is_alive()


requires_ffmpeg = pytest.mark.skipif(shutil.which("ffmpeg") is None, reason="ffmpeg not installed")
requires_videotoolbox = pytest.mark.skipif(
    encoder_failure(RenderConfig(encoder=VIDEOTOOLBOX)) is not None, reason="Apple media engine not usable"
)
EVERY_ENCODER = [pytest.param(X264), pytest.param(VIDEOTOOLBOX, marks=requires_videotoolbox)]


def small_video(path: Path, frames: int = 60, width: int = 320, height: int = 240) -> Path:
    write_video(path, [background(width, height, seed=i) for i in range(frames)], fps=30)
    return path
