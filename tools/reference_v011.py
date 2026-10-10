from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import math
import pickle
import platform
import re
import subprocess
import sys
import tempfile
from collections.abc import Generator, Iterable, Iterator
from contextlib import contextmanager, nullcontext
from dataclasses import asdict
from importlib.metadata import version
from pathlib import Path
from typing import Any, BinaryIO, TextIO, cast

import cv2
import numpy as np

import batdetect
from batdetect.bench.cli import main as bench_main
from batdetect.bench.collect import BenchRun
from batdetect.damage import frame_entries, parse_probe
from batdetect.detect import DetectConfig, Detection
from batdetect.exclusion import exclude
from batdetect.output.tables import write_tracks_csv
from batdetect.parallel import DEFAULT_WORKERS, detect_video
from batdetect.probe import PROBE_ARGS, ffprobe_version
from batdetect.spans import Span
from batdetect.stability import StabilityConfig
from batdetect.synthetic.injection import inject
from batdetect.synthetic.trajectory import SyntheticBat
from batdetect.track import Track, TrackConfig, track_detections
from batdetect.video import VideoInfo, open_video

REFERENCE_TAG = "v0.1.1"
FORMAT_VERSION = 1
REPO_ROOT = Path(__file__).resolve().parent.parent
DETECTION_FIELDS = ("x", "y", "left", "top", "width", "height", "area", "amplitude")
STREAM_ENTRIES = "stream=width,height,avg_frame_rate,r_frame_rate,time_base,nb_frames"
OPENCV_FFMPEG_LIBRARIES = ("avcodec", "avformat", "avutil", "swscale")
PATCH_BASE = 128
DECODER_ADDRESS = re.compile(r" @ 0x[0-9a-f]+\]")
GIT_DIFF_FOUND = 1


def check_reference_code() -> None:
    diff = subprocess.run(["git", "-C", str(REPO_ROOT), "diff", "--quiet", REFERENCE_TAG, "--", "src"], check=False)
    if diff.returncode not in {0, GIT_DIFF_FOUND}:
        sys.exit(f"cannot compare src/ with {REFERENCE_TAG}: is the tag fetched?")
    untracked = subprocess.run(
        ["git", "-C", str(REPO_ROOT), "status", "--porcelain", "--", "src"], capture_output=True, text=True, check=True
    )
    if diff.returncode == GIT_DIFF_FOUND or untracked.stdout:
        sys.exit(f"src/ differs from {REFERENCE_TAG}: the reference must be recorded with the code of {REFERENCE_TAG}")
    if not Path(batdetect.__file__).resolve().is_relative_to(REPO_ROOT / "src"):
        sys.exit(f"batdetect is imported from {batdetect.__file__}, not from {REPO_ROOT / 'src'}")


def dumps(value: object) -> str:
    return json.dumps(value, allow_nan=False, ensure_ascii=False, separators=(",", ":"))


@contextmanager
def replacing(path: Path) -> Generator[Path]:
    partial = path.with_name(path.name + ".partial")
    try:
        yield partial
        partial.replace(path)
    finally:
        partial.unlink(missing_ok=True)


@contextmanager
def text_output(path: Path, compressed: bool) -> Generator[TextIO]:
    with path.open("wb") as raw:
        packed = gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) if compressed else nullcontext(raw)
        with packed as sink:
            text = io.TextIOWrapper(cast(BinaryIO, sink), encoding="utf-8", newline="\n")
            yield text
            text.flush()
            text.detach()


def write_lines(path: Path, lines: Iterable[object], compressed: bool) -> None:
    with replacing(path) as partial, text_output(partial, compressed) as out:
        for line in lines:
            out.write(dumps(line) + "\n")


def write_json(path: Path, value: object) -> None:
    with replacing(path) as partial:
        partial.write_text(json.dumps(value, allow_nan=False, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def sha256(path: Path) -> str:
    with path.open("rb") as fh:
        return hashlib.file_digest(fh, "sha256").hexdigest()


def opencv_ffmpeg() -> dict[str, str]:
    found: dict[str, str] = {}
    for line in cv2.getBuildInformation().splitlines():
        name, _, value = line.strip().partition(":")
        if name in OPENCV_FFMPEG_LIBRARIES:
            found[name] = value.strip()
    return found


def versions() -> dict[str, object]:
    return {
        "batdetect": version("batdetect"),
        "python": platform.python_version(),
        "numpy": np.__version__,
        "opencv": cv2.__version__,
        "opencv_ffmpeg": opencv_ffmpeg(),
        "ffprobe": ffprobe_version(),
        "platform": platform.platform(),
    }


def stream_header(video: Path) -> dict[str, Any]:
    completed = subprocess.run(
        [
            "ffprobe",
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            STREAM_ENTRIES,
            "-of",
            "json",
            str(video),
        ],
        capture_output=True,
        check=True,
    )
    document = cast(dict[str, list[dict[str, Any]]], json.loads(completed.stdout))
    return document["streams"][0]


def detect_and_probe(video: Path, detect: DetectConfig, info: VideoInfo) -> tuple[dict[int, list[Detection]], str]:
    with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
        process = subprocess.Popen(
            ["ffprobe", *PROBE_ARGS, str(video)], stdout=output, stderr=errors, stdin=subprocess.DEVNULL
        )
        try:
            detections = detect_video(video, detect, info, DEFAULT_WORKERS)[0]
        except BaseException:
            process.kill()
            raise
        finally:
            process.wait()
        if process.returncode != 0:
            errors.seek(0)
            last = errors.read().decode(errors="replace")[-500:]
            sys.exit(f"ffprobe failed on {video} (exit code {process.returncode}) {last}")
        output.seek(0)
        return detections, output.read().decode(errors="replace")


def detection_fields(d: Detection) -> dict[str, float]:
    return {name: cast(float, getattr(d, name)) for name in DETECTION_FIELDS}


def frame_detections(frame: int, found: list[Detection]) -> list[dict[str, float]]:
    if any(d.frame != frame for d in found):
        sys.exit(f"a detection of frame {frame} carries another frame number")
    return [detection_fields(d) for d in found]


def frame_lines(entries: list[dict[str, object]], detections: dict[int, list[Detection]]) -> Iterator[object]:
    for index, ((frame, found), entry) in enumerate(zip(detections.items(), entries, strict=True)):
        if frame != index:
            sys.exit(f"frame {frame} of OpenCV is line {index} of ffprobe")
        pts = entry.get("pts")
        yield {
            "frame": frame,
            "pts": pts if isinstance(pts, int) else None,
            "key_frame": entry.get("key_frame") == 1,
            "decode_error": bool(entry.get("logs")),
            "detections": frame_detections(frame, found),
        }


def without_address(message: str) -> str:
    return DECODER_ADDRESS.sub("]", message.rstrip("\n"))


def decode_messages(entries: list[dict[str, object]]) -> list[dict[str, object]]:
    found: list[dict[str, object]] = []
    for frame, entry in enumerate(entries):
        logs = entry.get("logs")
        if isinstance(logs, list) and logs:
            messages = [cast(dict[str, object], log).get("message") for log in cast(list[object], logs)]
            found.append({"frame": frame, "messages": [without_address(str(m)) for m in messages]})
    return found


def track_lines(tracks: list[Track]) -> Iterator[object]:
    for t in tracks:
        yield {"id": t.id, "points": [{"frame": p.frame, **detection_fields(p)} for p in t.points]}


def span_pairs(spans: list[Span]) -> list[list[int]]:
    return [[s.first, s.last] for s in spans]


def record_data(video: Path, dest: Path, compressed: bool) -> None:
    detect, track, stability = DetectConfig(), TrackConfig(), StabilityConfig()
    cap, info = open_video(video, detect.work_width)
    cap.release()
    detections, probe_text = detect_and_probe(video, detect, info)
    entries = frame_entries(probe_text)
    probe = parse_probe(probe_text)
    margin = detect.half_window(info.fps)
    analysis = exclude(detections, probe, margin, info.fps, stability)
    tracks = track_detections(analysis.detections, track)
    dest.mkdir(parents=True, exist_ok=True)
    suffix = ".jsonl.gz" if compressed else ".jsonl"
    files = {"frames": f"frames{suffix}", "tracks": "tracks.jsonl", "csv": "tracks.csv"}
    write_lines(dest / files["frames"], frame_lines(entries, detections), compressed)
    write_lines(dest / files["tracks"], track_lines(tracks), compressed=False)
    with replacing(dest / files["csv"]) as partial:
        write_tracks_csv(tracks, info, partial)
    write_json(
        dest / "video.json",
        {
            "format_version": FORMAT_VERSION,
            "reference": REFERENCE_TAG,
            "video": {"name": video.name, "bytes": video.stat().st_size, "sha256": sha256(video)},
            "stream": stream_header(video),
            "opencv": {
                "fps": info.fps,
                "frame_count_property": info.frame_count,
                "frames_read": len(detections),
                "width": info.width,
                "height": info.height,
                "work_width": info.work_width,
                "work_height": info.work_height,
            },
            "ffprobe": {
                "frame_count": probe.frame_count,
                "error_frames": list(probe.error_frames),
                "gap_frames": list(probe.gap_frames),
                "decode_messages": decode_messages(entries),
            },
            "background_half_window": margin,
            "damaged_spans": span_pairs(analysis.damaged),
            "unstable_spans": span_pairs(analysis.unstable),
            "counts": {
                "frames": len(detections),
                "detections": sum(len(found) for found in detections.values()),
                "detections_kept": sum(len(found) for found in analysis.detections.values()),
                "damaged_frames_reported": analysis.reported_frames,
                "tracks": len(tracks),
                "track_points": sum(len(t.points) for t in tracks),
            },
            "config": {type(c).__name__: asdict(c) for c in (detect, track, stability)},
            "versions": versions(),
            "files": files,
        },
    )


def patch_window(bat: SyntheticBat, frame: int, width: int, height: int) -> tuple[int, int, int, int] | None:
    position = bat.position(frame)
    if position is None:
        return None
    x, y = position
    radius = math.ceil(3 * bat.sigma)
    x0, x1 = max(0, math.floor(x) - radius), min(width, math.floor(x) + radius + 1)
    y0, y1 = max(0, math.floor(y) - radius), min(height, math.floor(y) + radius + 1)
    return None if x0 >= x1 or y0 >= y1 else (x0, y0, x1, y1)


def gray_patch(bat: SyntheticBat, width: int, height: int) -> dict[str, object] | None:
    if abs(bat.amplitude) >= PATCH_BASE:
        sys.exit(f"target {bat.id} of amplitude {bat.amplitude} would be clipped on a uniform frame of {PATCH_BASE}")
    frame = bat.start_frame + len(bat.positions) // 2
    window = patch_window(bat, frame, width, height)
    if window is None:
        return None
    x0, y0, x1, y1 = window
    canvas = np.full((height, width, 3), PATCH_BASE, dtype=np.uint8)
    inject(canvas, [bat], frame)
    change = canvas[y0:y1, x0:x1, 0].astype(np.int16) - PATCH_BASE
    return {"frame": frame, "base": PATCH_BASE, "left": x0, "top": y0, "values": change.tolist()}


def bat_lines(run: BenchRun) -> Iterator[object]:
    for bat in run.bats:
        yield {
            "id": bat.id,
            "motion": bat.motion,
            "amplitude": bat.amplitude,
            "sigma": bat.sigma,
            "start_frame": bat.start_frame,
            "positions": [list(p) for p in bat.positions],
            "amplitudes": list(bat.amplitudes),
            "effective": [[o.frame, o.effective] for o in run.observations.get(bat.id, [])],
            "patch": gray_patch(bat, run.info.width, run.info.height),
        }


def injected_lines(run: BenchRun) -> Iterator[object]:
    for frame, found in run.injected.items():
        yield {"frame": frame, "detections": frame_detections(frame, found)}


def record_bench(video: Path, dest: Path, work: Path, bench_args: list[str]) -> None:
    if {"-o", "--out-dir"} & {arg.partition("=")[0] for arg in bench_args}:
        sys.exit("the bench writes into the work folder: do not pass -o or --out-dir")
    if (work / video.stem).exists():
        sys.exit(f"{work / video.stem} already exists: its cache could hold targets drawn with other settings")
    if bench_main([str(video), "-o", str(work), *bench_args]) != 0:
        sys.exit(f"batdetect-bench failed on {video}")
    with (work / video.stem / "cache.pkl").open("rb") as fh:
        run: object = pickle.load(fh)
    if not isinstance(run, BenchRun):
        sys.exit("the bench cache does not hold a bench run")
    report = cast(dict[str, object], json.loads((work / video.stem / "bench.json").read_text(encoding="utf-8")))
    report["code"] = REFERENCE_TAG
    report["video"] = video.name
    dest.mkdir(parents=True, exist_ok=True)
    write_json(dest / "bench.json", report)
    write_lines(dest / "bats.jsonl.gz", bat_lines(run), compressed=True)
    write_lines(dest / "injected.jsonl.gz", injected_lines(run), compressed=True)
    write_json(
        dest / "bench-run.json",
        {
            "format_version": FORMAT_VERSION,
            "reference": REFERENCE_TAG,
            "video": {"name": video.name, "sha256": sha256(video)},
            "arguments": bench_args,
            "counts": {
                "bats": len(run.bats),
                "frames": len(run.injected),
                "injected_detections": sum(len(found) for found in run.injected.values()),
            },
            "versions": versions(),
            "files": {"report": "bench.json", "bats": "bats.jsonl.gz", "injected": "injected.jsonl.gz"},
        },
    )


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(description=f"Record what batdetect {REFERENCE_TAG} finds, for the Rust rewrite.")
    commands = ap.add_subparsers(dest="command", required=True)
    data = commands.add_parser("data", help="frames, raw detections, spans and tracks of one video")
    data.add_argument("video", type=Path)
    data.add_argument("dest", type=Path)
    data.add_argument("--gzip", action="store_true", help="compress the frame lines")
    bench = commands.add_parser("bench", help="targets, injected detections and report of one bench")
    bench.add_argument("video", type=Path)
    bench.add_argument("dest", type=Path)
    bench.add_argument("work", type=Path, help="folder for the bench cache, kept out of the reference")
    bench.add_argument("bench_args", nargs=argparse.REMAINDER, help="arguments passed to batdetect-bench")
    return ap


def main(argv: list[str] | None = None) -> int:
    ns = build_parser().parse_args(argv)
    check_reference_code()
    if ns.command == "data":
        record_data(ns.video, ns.dest, ns.gzip)
    else:
        record_bench(ns.video, ns.dest, ns.work, ns.bench_args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
