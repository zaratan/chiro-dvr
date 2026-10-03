from __future__ import annotations

import csv
import json
import shutil
import subprocess
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Any

import cv2
import numpy as np

from batdetect.pipeline import ColorFrame, Detection, Track, VideoError, VideoInfo

if TYPE_CHECKING:
    from _typeshed import DataclassInstance

BOX_COLOR = (0, 0, 255)
TRAIL_COLOR = (0, 200, 255)
SUMMARY_FRAME = 30
MAX_CRF = 51


@dataclass(frozen=True, slots=True)
class RenderConfig:
    box_pad: int = 12
    trail_s: float = 1.0
    clip_margin_s: float = 1.5
    crf: int = 20

    def __post_init__(self) -> None:
        if self.box_pad < 0:
            raise ValueError("box_pad must be >= 0")
        if self.trail_s < 0:
            raise ValueError("trail_s must be >= 0")
        if self.clip_margin_s < 0:
            raise ValueError("clip_margin_s must be >= 0")
        if not 0 <= self.crf <= MAX_CRF:
            raise ValueError(f"crf must be between 0 and {MAX_CRF}")


def format_time(seconds: float) -> str:
    centis = round(seconds * 100)
    minutes, rest = divmod(centis, 6000)
    return f"{minutes}:{rest / 100:05.2f}"


def clip_name(track: Track, fps: float) -> str:
    stamp = format_time(track.first.frame / fps).replace(":", "m").replace(".", "s")
    return f"{track.id:02d}_{stamp}.mp4"


def _box(det: Detection, scale: float, pad: int) -> tuple[tuple[int, int], tuple[int, int]]:
    x0 = int(det.left * scale) - pad
    y0 = int(det.top * scale) - pad
    x1 = int((det.left + det.width) * scale) + pad
    y1 = int((det.top + det.height) * scale) + pad
    return (x0, y0), (x1, y1)


def draw_overlay(
    frame: ColorFrame, hits: list[tuple[Track, Detection]], info: VideoInfo, cfg: RenderConfig, frame_no: int
) -> None:
    trail_frames = round(cfg.trail_s * info.fps)
    for track, det in hits:
        top_left, bottom_right = _box(det, info.scale, cfg.box_pad)
        cv2.rectangle(frame, top_left, bottom_right, BOX_COLOR, 2)
        label_at = (top_left[0], top_left[1] - 6)
        cv2.putText(frame, f"#{track.id}", label_at, cv2.FONT_HERSHEY_SIMPLEX, 0.8, BOX_COLOR, 2)
        trail = [p for p in track.points if frame_no - trail_frames <= p.frame <= frame_no]
        if len(trail) > 1:
            pts = np.array([(round(p.x * info.scale), round(p.y * info.scale)) for p in trail], dtype=np.int32)
            cv2.polylines(frame, [pts], isClosed=False, color=TRAIL_COLOR, thickness=2)


def render_annotated(video: Path, tracks: list[Track], info: VideoInfo, out_path: Path, cfg: RenderConfig) -> None:
    by_frame: dict[int, list[tuple[Track, Detection]]] = {}
    for track in tracks:
        for det in track.points:
            by_frame.setdefault(det.frame, []).append((track, det))
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


def summary_image(video: Path, tracks: list[Track], info: VideoInfo, out_path: Path) -> None:
    cap = cv2.VideoCapture(str(video))
    cap.set(cv2.CAP_PROP_POS_FRAMES, min(SUMMARY_FRAME, max(0, info.frame_count - 1)))
    ok, raw = cap.read()
    cap.release()
    if not ok:
        raise VideoError(f"cannot read a background frame from {video}")
    image = (np.asarray(raw, dtype=np.float64) * 0.6).astype(np.uint8)
    for track in tracks:
        pts = np.array([(round(p.x * info.scale), round(p.y * info.scale)) for p in track.points], dtype=np.int32)
        cv2.polylines(image, [pts], isClosed=False, color=TRAIL_COLOR, thickness=3)
        start = (int(pts[0][0]) + 6, int(pts[0][1]))
        label = f"#{track.id} {format_time(track.first.frame / info.fps)}"
        cv2.putText(image, label, start, cv2.FONT_HERSHEY_SIMPLEX, 0.9, BOX_COLOR, 2)
    if not cv2.imwrite(str(out_path), image):
        raise VideoError(f"cannot write {out_path}")


def write_tracks_csv(tracks: list[Track], info: VideoInfo, out_path: Path) -> None:
    with out_path.open("w", newline="") as fh:
        writer = csv.writer(fh)
        writer.writerow(
            [
                "id",
                "start",
                "end",
                "start_s",
                "duration_s",
                "hits",
                "chord_px",
                "path_px",
                "speed_px_s",
                "max_area_px",
                "peak_amplitude",
            ]
        )
        for t in tracks:
            start = t.first.frame / info.fps
            duration = max((t.last.frame - t.first.frame) / info.fps, 1 / info.fps)
            path_px = t.path_length() * info.scale
            area_scale = info.scale**2
            writer.writerow(
                [
                    t.id,
                    format_time(start),
                    format_time(t.last.frame / info.fps),
                    round(start, 2),
                    round(duration, 2),
                    len(t.points),
                    round(t.chord() * info.scale, 1),
                    round(path_px, 1),
                    round(path_px / duration, 1),
                    round(max(p.area for p in t.points) * area_scale),
                    round(max(p.amplitude for p in t.points), 1),
                ]
            )


def write_params(params: dict[str, Any], out_path: Path) -> None:
    out_path.write_text(json.dumps(params, indent=2, ensure_ascii=False) + "\n")


def config_params(*configs: DataclassInstance) -> dict[str, Any]:
    return {type(c).__name__: asdict(c) for c in configs}


def split_clips(annotated: Path, tracks: list[Track], info: VideoInfo, split_dir: Path, cfg: RenderConfig) -> None:
    shutil.rmtree(split_dir, ignore_errors=True)
    split_dir.mkdir(parents=True)
    duration = info.frame_count / info.fps
    for track in tracks:
        start = max(0.0, track.first.frame / info.fps - cfg.clip_margin_s)
        end = min(duration, track.last.frame / info.fps + cfg.clip_margin_s)
        command = [
            "ffmpeg",
            "-v",
            "error",
            "-y",
            "-ss",
            f"{start:.3f}",
            "-i",
            str(annotated),
            "-t",
            f"{end - start:.3f}",
            "-c:v",
            "libx264",
            "-crf",
            str(cfg.crf),
            "-pix_fmt",
            "yuv420p",
            str(split_dir / clip_name(track, info.fps)),
        ]
        if subprocess.run(command, check=False).returncode != 0:
            raise VideoError(f"ffmpeg failed while cutting clip #{track.id}")
