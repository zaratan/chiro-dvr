from __future__ import annotations

from pathlib import Path

import cv2
import numpy as np

from batdetect.detect import DetectConfig, osd_mask
from batdetect.output.style import PANEL_COLOR
from batdetect.video import ColorFrame, VideoError, VideoInfo

BACKGROUND_SAMPLES = 25


def median_background(video: Path, info: VideoInfo, samples: int = BACKGROUND_SAMPLES) -> ColorFrame:
    cap = cv2.VideoCapture(str(video))
    if not cap.isOpened():
        raise VideoError(f"cannot open {video}")
    count = max(1, min(samples, info.frame_count))
    frames: list[ColorFrame] = []
    for i in range(count):
        cap.set(cv2.CAP_PROP_POS_FRAMES, (2 * i + 1) * info.frame_count // (2 * count))
        ok, frame = cap.read()
        if ok:
            frames.append(np.asarray(frame, dtype=np.uint8))
    cap.release()
    if not frames:
        raise VideoError(f"cannot read a background frame from {video}")
    return np.asarray(np.median(np.stack(frames), axis=0), dtype=np.float64).round().astype(np.uint8)


def hide_display(background: ColorFrame, cfg: DetectConfig) -> ColorFrame:
    height, width = background.shape[:2]
    hidden = background.copy()
    hidden[~osd_mask(width, height, cfg)] = PANEL_COLOR
    return hidden
