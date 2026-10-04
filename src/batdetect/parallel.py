from __future__ import annotations

import multiprocessing
import os
import threading
from collections.abc import Callable, Iterator
from concurrent.futures import ProcessPoolExecutor
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Protocol

import cv2

from batdetect.detect import DetectConfig, Detection, detect_frames
from batdetect.prefetch import prefetched
from batdetect.video import ColorFrame, GrayFrame, VideoInfo, open_video, read_frames, to_work_gray

MIN_CHUNK_WINDOWS = 10
READ_AHEAD = 4
DEFAULT_WORKERS = 1


class FrameHook(Protocol):
    def __call__(self, frame: ColorFrame, frame_no: int) -> None: ...


@dataclass(frozen=True, slots=True)
class Chunk:
    start: int
    stop: int | None

    def owns(self, frame: int) -> bool:
        return self.start <= frame and (self.stop is None or frame < self.stop)


def exit_with_parent() -> None:
    parent = multiprocessing.parent_process()
    if parent is None:
        return

    def watch() -> None:
        parent.join()
        os._exit(1)

    threading.Thread(target=watch, daemon=True).start()


def plan_chunks(frame_count: int, workers: int, min_length: int) -> list[Chunk]:
    count = max(1, min(workers, frame_count // max(1, min_length)))
    bounds = [round(k * frame_count / count) for k in range(count + 1)]
    return [Chunk(bounds[k], bounds[k + 1] if k < count - 1 else None) for k in range(count)]


def skip_frames(cap: cv2.VideoCapture, count: int) -> None:
    for _ in range(count):
        if not cap.grab():
            return


def detect_chunk[H: FrameHook](
    video: Path, cfg: DetectConfig, chunk: Chunk, hook: H | None
) -> tuple[dict[int, list[Detection]], H | None]:
    cap, info = open_video(video, cfg.work_width)
    half = cfg.half_window(info.fps)
    first = max(0, chunk.start - half)
    last = None if chunk.stop is None else chunk.stop + half

    def frames() -> Iterator[GrayFrame]:
        for k, frame in enumerate(read_frames(cap)):
            frame_no = first + k
            if last is not None and frame_no >= last:
                return
            if hook is not None:
                hook(frame, frame_no)
            yield to_work_gray(frame, info)

    try:
        skip_frames(cap, first)
        with prefetched(frames(), READ_AHEAD) as grays:
            local = detect_frames(grays, info.fps, cfg, info.scale)
    finally:
        cap.release()
    owned = {
        first + k: [replace(d, frame=first + k) for d in found] for k, found in local.items() if chunk.owns(first + k)
    }
    return owned, hook


def detect_video[H: FrameHook](
    video: Path,
    cfg: DetectConfig,
    info: VideoInfo,
    workers: int,
    make_hook: Callable[[], H] | None = None,
) -> tuple[dict[int, list[Detection]], list[H]]:
    chunks = plan_chunks(info.frame_count, workers, MIN_CHUNK_WINDOWS * 2 * cfg.half_window(info.fps))
    hooks = [make_hook() if make_hook is not None else None for _ in chunks]
    if len(chunks) == 1:
        results = [detect_chunk(video, cfg, chunks[0], hooks[0])]
    else:
        context = multiprocessing.get_context("spawn")
        with ProcessPoolExecutor(max_workers=len(chunks), mp_context=context, initializer=exit_with_parent) as pool:
            futures = [pool.submit(detect_chunk, video, cfg, c, h) for c, h in zip(chunks, hooks, strict=True)]
            try:
                results = [f.result() for f in futures]
            except BaseException:
                pool.shutdown(wait=False, cancel_futures=True)
                raise
    detections: dict[int, list[Detection]] = {}
    returned: list[H] = []
    for found, hook in results:
        detections.update(found)
        if hook is not None:
            returned.append(hook)
    return dict(sorted(detections.items())), returned
