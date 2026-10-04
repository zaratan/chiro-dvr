from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import cv2
import numpy as np
import numpy.typing as npt

GrayFrame = npt.NDArray[np.uint8]
ColorFrame = npt.NDArray[np.uint8]


class VideoError(Exception):
    pass


@dataclass(frozen=True, slots=True)
class VideoInfo:
    fps: float
    frame_count: int
    width: int
    height: int
    work_width: int
    work_height: int

    @property
    def scale(self) -> float:
        return self.width / self.work_width


def open_video(path: Path, work_width: int) -> tuple[cv2.VideoCapture, VideoInfo]:
    cap = cv2.VideoCapture(str(path))
    if not cap.isOpened():
        raise VideoError(f"cannot open {path}")
    width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    fps = cap.get(cv2.CAP_PROP_FPS)
    if width <= 0 or height <= 0 or fps <= 0:
        cap.release()
        raise VideoError(f"{path} has no readable video stream")
    work_width = min(work_width, width)
    work_height = max(1, round(work_width * height / width))
    info = VideoInfo(fps, int(cap.get(cv2.CAP_PROP_FRAME_COUNT)), width, height, work_width, work_height)
    return cap, info


def read_frames(cap: cv2.VideoCapture) -> Iterator[ColorFrame]:
    while True:
        ok, frame = cap.read()
        if not ok:
            return
        yield np.asarray(frame, dtype=np.uint8)


def to_work_gray(frame: ColorFrame, info: VideoInfo) -> GrayFrame:
    if (info.work_width, info.work_height) != (info.width, info.height):
        frame = np.asarray(
            cv2.resize(frame, (info.work_width, info.work_height), interpolation=cv2.INTER_AREA), dtype=np.uint8
        )
    return np.asarray(cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY), dtype=np.uint8)


def read_gray_frames(cap: cv2.VideoCapture, info: VideoInfo) -> Iterator[GrayFrame]:
    return (to_work_gray(frame, info) for frame in read_frames(cap))
