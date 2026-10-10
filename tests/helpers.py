from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import threading
from collections.abc import Callable, Iterable
from pathlib import Path

import cv2
import numpy as np
import pytest

from batdetect.detect import DetectConfig, Detection
from batdetect.exclusion import exclude
from batdetect.output.config import VIDEOTOOLBOX, X264, RenderConfig
from batdetect.output.encoder import encoder_failure
from batdetect.parallel import detect_video
from batdetect.probe import probing
from batdetect.stability import StabilityConfig
from batdetect.track import Track, TrackConfig, track_detections
from batdetect.video import FFMPEG_LOG_LEVEL, GrayFrame, open_video

BACKGROUND = 200
GUARD_TIMEOUT_S = 5.0
NOISE = 3
START_TOLERANCE_FRAMES = 5
HITS_TOLERANCE = 3
START_CODE = b"\x00\x00\x01"
SLICE_TYPES = (1, 5)
NAL_TYPE_MASK = 0x1F
CUT_FRACTION = 1 / 3
SYNTHETIC_FRAMES = 120
SYNTHETIC_GOP = 15
DAMAGED_PICTURES = {20, 21, 50}
DECODER_ERROR = "error while decoding"


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
    with probing(video) as outcome:
        detections = detect_video(video, detect, info, workers=1)[0]
    analysis = exclude(detections, outcome.probe, detect.half_window(info.fps), info.fps, StabilityConfig())
    return track_detections(analysis.detections, TrackConfig()), info.fps


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


requires_ffmpeg = pytest.mark.skipif(
    shutil.which("ffmpeg") is None or shutil.which("ffprobe") is None, reason="ffmpeg or ffprobe not installed"
)
requires_videotoolbox = pytest.mark.skipif(
    encoder_failure(RenderConfig(encoder=VIDEOTOOLBOX)) is not None, reason="Apple media engine not usable"
)
EVERY_ENCODER = [pytest.param(X264), pytest.param(VIDEOTOOLBOX, marks=requires_videotoolbox)]


def small_video(path: Path, frames: int = 60, width: int = 320, height: int = 240) -> Path:
    write_video(path, [background(width, height, seed=i) for i in range(frames)], fps=30)
    return path


def nal_units(stream: bytes) -> list[bytes]:
    starts: list[int] = []
    at = stream.find(START_CODE)
    while at >= 0:
        starts.append(at + len(START_CODE))
        at = stream.find(START_CODE, at + len(START_CODE))
    ends = [s - len(START_CODE) for s in starts[1:]] + [len(stream)]
    return [stream[s:e].rstrip(b"\x00") for s, e in zip(starts, ends, strict=True)]


def cut_slices(stream: bytes, pictures: set[int], fraction: float) -> bytes:
    out = bytearray()
    picture = -1
    for unit in nal_units(stream):
        is_slice = unit[0] & NAL_TYPE_MASK in SLICE_TYPES
        picture += int(is_slice)
        cut = is_slice and picture in pictures
        kept = unit[: len(unit) - max(1, int(len(unit) * fraction))] if cut else unit
        out += b"\x00" + START_CODE + kept
    return bytes(out)


def damaged_video(path: Path, pictures: set[int], fraction: float = CUT_FRACTION) -> Path:
    raw = path.with_suffix(".h264")
    subprocess.run(
        [
            "ffmpeg",
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            f"testsrc2=size=320x240:rate=30:duration={SYNTHETIC_FRAMES / 30}",
            "-c:v",
            "libx264",
            "-g",
            str(SYNTHETIC_GOP),
            "-bf",
            "0",
            "-pix_fmt",
            "yuv420p",
            "-x264-params",
            "slices=1:scenecut=0",
            "-threads",
            "1",
            "-bsf:v",
            "h264_mp4toannexb",
            "-f",
            "h264",
            str(raw),
        ],
        check=True,
    )
    raw.write_bytes(cut_slices(raw.read_bytes(), pictures, fraction))
    subprocess.run(["ffmpeg", "-v", "quiet", "-y", "-r", "30", "-i", str(raw), "-c", "copy", str(path)], check=True)
    raw.unlink()
    return path


def decoder_log(video: Path, *call: str) -> str:
    script = "\n".join(
        [
            "from pathlib import Path",
            "from batdetect.video import VideoInfo",
            f"video = Path({str(video)!r})",
            f"info = VideoInfo(30.0, {SYNTHETIC_FRAMES}, 320, 240, 160, 120)",
            *call,
        ]
    )
    env = {k: v for k, v in os.environ.items() if k != FFMPEG_LOG_LEVEL}
    done = subprocess.run([sys.executable, "-c", script], env=env, capture_output=True, text=True, check=False)
    assert done.returncode == 0, done.stderr
    return done.stdout + done.stderr


def noisy_damaged_video(path: Path) -> Path:
    video = damaged_video(path, DAMAGED_PICTURES)
    assert DECODER_ERROR in decoder_log(
        video, "import cv2", "cap = cv2.VideoCapture(str(video))", "while cap.grab(): pass"
    )
    return video


ANSI_STYLE = re.compile(r"\x1b\[[0-9;]*m")
OPTION_ENTRY = re.compile(r"^  (-.+?)(?=^  -|^\S|\Z)", re.MULTILINE | re.DOTALL)


def option_help(help_text: str) -> dict[str, str]:
    entries = OPTION_ENTRY.findall(ANSI_STYLE.sub("", help_text))
    return {entry.split()[0].rstrip(","): " ".join(entry.split()) for entry in entries}
