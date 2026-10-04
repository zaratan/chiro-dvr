from __future__ import annotations

import shutil
import sys
from collections.abc import Callable
from contextlib import ExitStack
from dataclasses import dataclass
from pathlib import Path

import cv2
import numpy as np

from batdetect.output.clips import ClipWindow, in_passes
from batdetect.output.config import RenderConfig
from batdetect.output.writer import FrameWriter
from batdetect.video import ColorFrame, VideoError, VideoInfo

MAX_WRITERS = 6
WHOLE_VIDEO = sys.maxsize

type Draw = Callable[[ColorFrame, int], None]


@dataclass(frozen=True, slots=True)
class VideoOutputs:
    split_dir: Path
    clips: list[ClipWindow]
    annotated: Path


def render_videos(video: Path, info: VideoInfo, outputs: VideoOutputs, draw: Draw, cfg: RenderConfig) -> None:
    outputs.annotated.unlink(missing_ok=True)
    shutil.rmtree(outputs.split_dir, ignore_errors=True)
    outputs.split_dir.mkdir(parents=True)
    windows = list(outputs.clips)
    if cfg.annotated:
        windows.append(ClipWindow(0, WHOLE_VIDEO, outputs.annotated))
    for batch in in_passes(windows, MAX_WRITERS):
        render_pass(video, info, batch, draw, cfg)


def render_pass(video: Path, info: VideoInfo, windows: list[ClipWindow], draw: Draw, cfg: RenderConfig) -> None:
    starting: dict[int, list[ClipWindow]] = {}
    for window in windows:
        starting.setdefault(window.first, []).append(window)
    end = max(w.last for w in windows)
    cap = cv2.VideoCapture(str(video))
    if not cap.isOpened():
        raise VideoError(f"cannot open {video}")
    try:
        with ExitStack() as stack:
            active: dict[ClipWindow, FrameWriter] = {}
            frame_no = 0
            while frame_no <= end:
                for window in starting.get(frame_no, []):
                    active[window] = stack.enter_context(FrameWriter(window.path, info, cfg))
                if not active:
                    if not cap.grab():
                        break
                else:
                    ok, raw = cap.read()
                    if not ok:
                        break
                    frame = np.asarray(raw, dtype=np.uint8)
                    draw(frame, frame_no)
                    data = frame.tobytes()
                    for writer in active.values():
                        writer.write(data)
                    for window in [w for w in active if w.last == frame_no]:
                        active.pop(window).close()
                frame_no += 1
    finally:
        cap.release()
