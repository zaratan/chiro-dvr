from __future__ import annotations

import subprocess
from pathlib import Path

import cv2
import numpy as np

from batdetect.detect import Detection
from batdetect.output.config import RenderConfig
from batdetect.output.overlay import draw_overlay, filled_points
from batdetect.track import Track
from batdetect.video import VideoError, VideoInfo


def render_annotated(video: Path, tracks: list[Track], info: VideoInfo, out_path: Path, cfg: RenderConfig) -> None:
    by_frame: dict[int, list[tuple[Track, Detection, bool]]] = {}
    for track in tracks:
        for det, interpolated in filled_points(track):
            by_frame.setdefault(det.frame, []).append((track, det, interpolated))
    cap = cv2.VideoCapture(str(video))
    if not cap.isOpened():
        raise VideoError(f"cannot open {video}")
    command = [
        "ffmpeg",
        "-v",
        "error",
        "-y",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "bgr24",
        "-s",
        f"{info.width}x{info.height}",
        "-r",
        str(info.fps),
        "-i",
        "-",
        "-c:v",
        "libx264",
        "-crf",
        str(cfg.crf),
        "-pix_fmt",
        "yuv420p",
        str(out_path),
    ]
    ffmpeg = subprocess.Popen(command, stdin=subprocess.PIPE)
    stdin = ffmpeg.stdin
    if stdin is None:
        raise VideoError("ffmpeg stdin unavailable")
    try:
        frame_no = 0
        while True:
            ok, raw = cap.read()
            if not ok:
                break
            frame = np.asarray(raw, dtype=np.uint8)
            draw_overlay(frame, by_frame.get(frame_no, []), info, cfg, frame_no)
            stdin.write(frame.tobytes())
            frame_no += 1
    except BrokenPipeError:
        pass
    finally:
        cap.release()
        stdin.close()
    if ffmpeg.wait() != 0:
        raise VideoError(f"ffmpeg failed while writing {out_path}")
