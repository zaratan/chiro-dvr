"""Run batdetect with detection and damage scan done by the Rust `fastdet` binary in one decoding pass.

uv run python rust-proto/fast_run.py VIDEO -o OUT [batdetect options]
Tracking, exclusion, tables, summary and clips stay in Python, unchanged.
"""

from __future__ import annotations

import os
import queue
import subprocess
import sys
import tempfile
import threading
import time
from collections.abc import Generator
from concurrent.futures import ThreadPoolExecutor
from contextlib import ExitStack, contextmanager
from pathlib import Path

import numpy as np

from batdetect import cli
from batdetect.damage import Probe, damaged_spans, frames_after_pts_gaps
from batdetect.detect import Detection
from batdetect.output import render
from batdetect.output.clips import ClipWindow
from batdetect.output.writer import FrameWriter
from batdetect.video import VideoError, VideoInfo

HERE = Path(__file__).parent
FASTDET = Path(os.environ.get("FASTDET", HERE / "target" / "fast-dev" / "release" / "fastdet"))
FRAMESRC = HERE / "target" / "fast-dev" / "release" / "framesrc"
SEQUENTIAL = HERE / "target" / "fast-dev" / "release" / "fastdet"
SLICES_UNSAFE = 3
NO_PTS = -(2**63)
PTS_BYTES = 8
REPLAY_MARGIN = 30
TIMINGS: dict[str, float] = {}


def run_fastdet(video: Path) -> tuple[dict[int, list[Detection]], Probe]:
    with tempfile.TemporaryDirectory() as tmp:
        found_path, frames_path = Path(tmp) / "detections.txt", Path(tmp) / "frames.txt"
        t0 = time.perf_counter()
        completed = subprocess.run(
            [str(FASTDET), str(video), str(found_path), str(frames_path)], capture_output=True, text=True, check=False
        )
        if completed.returncode == SLICES_UNSAFE:
            print(completed.stderr.strip(), "-> sequential decoding", file=sys.stderr)
            completed = subprocess.run(
                [str(SEQUENTIAL), str(video), str(found_path), str(frames_path)],
                capture_output=True,
                text=True,
                check=True,
            )
        completed.check_returncode()
        TIMINGS["fastdet"] = time.perf_counter() - t0
        print(completed.stderr.strip(), file=sys.stderr)
        frames = [line.split() for line in frames_path.read_text().splitlines()]
        detections: dict[int, list[Detection]] = {int(f[0]): [] for f in frames}
        for line in found_path.read_text().splitlines():
            f = line.split()
            frame = int(f[0])
            x, y, left, top, width, height, area, amplitude = map(float, f[1:])
            detections[frame].append(Detection(frame, x, y, left, top, width, height, area, amplitude))
    pts = [None if f[1] == "-" else int(f[1]) for f in frames]
    CURRENT.pts = pts
    probe = Probe(
        frame_count=len(frames),
        error_frames=tuple(int(f[0]) for f in frames if int(f[4]) != 0),
        gap_frames=tuple(frames_after_pts_gaps(pts)),
        key_frames=tuple(int(f[0]) for f in frames if f[2] == "1"),
    )
    return detections, probe


class Outcome:
    def __init__(self) -> None:
        self.probe: Probe | None = None
        self.pts: list[int | None] | None = None


@contextmanager
def in_process_probing(_video: Path) -> Generator[Outcome]:
    yield CURRENT


CURRENT = Outcome()


def fast_detect_video(video, cfg, info, workers, make_hook=None):
    if os.environ.get("FAST_CHECK_DEFAULTS", "1") == "1" and cfg != cli.DetectConfig():
        raise SystemExit("fastdet implements the default detection settings only")
    detections, probe = run_fastdet(video)
    CURRENT.probe = probe
    return detections, []


def frame_pts(video: Path) -> list[int | None]:
    if CURRENT.pts is None:
        listed = subprocess.run([str(FRAMESRC), str(video), "--pts"], capture_output=True, text=True, check=True)
        CURRENT.pts = [None if line == str(NO_PTS) else int(line) for line in listed.stdout.split()]
    return CURRENT.pts


def replay_from(frame: int) -> int:
    probe = CURRENT.probe
    if probe is None:
        return frame
    for span in damaged_spans(probe, REPLAY_MARGIN):
        if span.first <= frame <= span.last:
            return span.first
    return frame


def served_frames(video: Path, info: VideoInfo, ranges: list[tuple[int, int]]):
    pts = frame_pts(video)
    if any(pts[k] is None for a, b in ranges for k in (a, b)):
        raise VideoError(f"{video}: a clip starts or ends on a frame without pts")
    numbers = {p: k for k, p in enumerate(pts) if p is not None}
    expected = (k for a, b in ranges for k in range(a, b + 1))
    size = info.width * info.height * 3
    command = [str(FRAMESRC), str(video), *[f"{pts[replay_from(a)]}:{pts[a]}:{pts[b]}" for a, b in ranges]]
    with subprocess.Popen(command, stdout=subprocess.PIPE) as proc:
        assert proc.stdout is not None
        buf = bytearray(PTS_BYTES + size)
        view = memoryview(buf)
        for want in expected:
            got = 0
            while got < len(buf):
                n = proc.stdout.readinto(view[got:])
                if not n:
                    raise VideoError(f"{video}: frame {want} was not served")
                got += n
            served = numbers.get(int.from_bytes(buf[:PTS_BYTES], "little", signed=True))
            if served != want:
                raise VideoError(f"{video}: served frame {served} where frame {want} was expected")
            yield want, np.frombuffer(view[PTS_BYTES:], dtype=np.uint8).reshape(info.height, info.width, 3)


def merged_ranges(spans: list[tuple[int, int]]) -> list[tuple[int, int]]:
    merged: list[tuple[int, int]] = []
    for a, b in sorted(spans):
        if merged and a <= merged[-1][1] + 1:
            merged[-1] = (merged[-1][0], max(merged[-1][1], b))
        else:
            merged.append((a, b))
    return merged


class ThreadedWriter:
    def __init__(self, inner: FrameWriter) -> None:
        self.inner = inner
        self.queue: queue.Queue[bytes | None] = queue.Queue(maxsize=8)
        self.thread = threading.Thread(target=self.drain, daemon=True)
        self.thread.start()

    def drain(self) -> None:
        while (data := self.queue.get()) is not None:
            self.inner.write(data)

    def write(self, data: bytes) -> None:
        self.queue.put(data)

    def close(self) -> None:
        self.queue.put(None)
        self.thread.join()
        self.inner.close()

    def __enter__(self) -> ThreadedWriter:
        return self

    def __exit__(self, *exc: object) -> None:
        self.close()


def fast_render_pass(video: Path, info: VideoInfo, windows: list[ClipWindow], draw, cfg) -> None:
    t0 = time.perf_counter()
    last = info.frame_count - 1
    starting: dict[int, list[ClipWindow]] = {}
    for window in windows:
        starting.setdefault(window.first, []).append(window)
    ranges = merged_ranges([(w.first, min(w.last, last)) for w in windows])
    with ExitStack() as stack:
        active: dict[ClipWindow, FrameWriter] = {}
        for frame_no, frame in served_frames(video, info, ranges):
            for window in starting.get(frame_no, []):
                writer = stack.enter_context(FrameWriter(window.path, info, cfg, window.slowdown))
                active[window] = ThreadedWriter(writer)
            render.write_frame(frame, frame_no, active, draw)
            for window in [w for w in active if min(w.last, last) == frame_no]:
                active.pop(window).close()
    TIMINGS["render"] = TIMINGS.get("render", 0) + time.perf_counter() - t0


def render_window(video: Path, info: VideoInfo, window: ClipWindow, draw, cfg) -> None:
    last = min(window.last, info.frame_count - 1)
    with FrameWriter(window.path, info, cfg, window.slowdown) as writer:
        active = {window: writer}
        for frame_no, frame in served_frames(video, info, [(window.first, last)]):
            render.write_frame(frame, frame_no, active, draw)


def parallel_render_pass(video: Path, info: VideoInfo, windows: list[ClipWindow], draw, cfg) -> None:
    t0 = time.perf_counter()
    jobs = int(os.environ.get("FAST_RENDER_JOBS", "4"))
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        for future in [pool.submit(render_window, video, info, w, draw, cfg) for w in windows]:
            future.result()
    TIMINGS["render"] = TIMINGS.get("render", 0) + time.perf_counter() - t0


def fast_median_background(video: Path, info: VideoInfo, samples: int = 25):
    count = max(1, min(samples, info.frame_count))
    positions = [(2 * i + 1) * info.frame_count // (2 * count) for i in range(count)]
    frames = [f.copy() for _, f in served_frames(video, info, [(p, p) for p in positions])]
    if not frames:
        raise VideoError(f"cannot read a background frame from {video}")
    return np.asarray(np.median(np.stack(frames), axis=0), dtype=np.float64).round().astype(np.uint8)


python_probing = cli.probing


@contextmanager
def remembered_probing(video: Path) -> Generator[object]:
    with python_probing(video) as outcome:
        yield outcome
    CURRENT.probe = outcome.probe


if os.environ.get("FAST_DETECT", "1") == "1":
    cli.detect_video = fast_detect_video
    cli.probing = in_process_probing
else:
    cli.probing = remembered_probing
if os.environ.get("FAST_RENDER", "1") == "1":
    render.render_pass = parallel_render_pass if os.environ.get("FAST_RENDER_JOBS", "4") != "1" else fast_render_pass
    cli.median_background = fast_median_background

if __name__ == "__main__":
    t0 = time.perf_counter()
    code = cli.main()
    total = time.perf_counter() - t0
    print(
        f"timings: fastdet {TIMINGS.get('fastdet', 0):.2f} s, render {TIMINGS.get('render', 0):.2f} s, total {total:.2f} s",
        file=sys.stderr,
    )
    sys.exit(code)
