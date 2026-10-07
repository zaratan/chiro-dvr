from __future__ import annotations

import argparse
import json
import shutil
import sys
from dataclasses import asdict
from pathlib import Path

from batdetect.arguments import (
    add_detection_arguments,
    add_tracking_arguments,
    build_detect_config,
    build_stability_config,
    build_track_config,
    positive_int,
)
from batdetect.bench.cache import load_or_collect
from batdetect.bench.config import DEFAULT_AMPLITUDES, DEFAULT_SIGMAS, BenchSetup, MatchConfig, bench_classes
from batdetect.bench.metrics import evaluate
from batdetect.bench.report import code_version, format_table
from batdetect.bench.stats import summarize
from batdetect.parallel import DEFAULT_WORKERS
from batdetect.synthetic.sampling import Sampling
from batdetect.synthetic.trajectory import MOTIONS, Motion
from batdetect.video import VideoError


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(prog="batdetect-bench", description="Measure detection on injected synthetic bats.")
    ap.add_argument("video", type=Path)
    ap.add_argument("-o", "--out-dir", type=Path, default=Path("out/bench"))
    ap.add_argument("--per-class", type=int, default=30)
    ap.add_argument("--amplitudes", type=float, nargs="+", default=list(DEFAULT_AMPLITUDES))
    ap.add_argument("--sigmas", type=float, nargs="+", default=list(DEFAULT_SIGMAS), help="blob size, source pixels")
    ap.add_argument(
        "--motions", nargs="+", choices=MOTIONS, default=["pass"], help="pass: smooth crossing; hunt, circle: turns"
    )
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--workers", type=positive_int, default=DEFAULT_WORKERS)
    ap.add_argument("--match-radius", type=float, default=MatchConfig().radius, help="source pixels")
    add_detection_arguments(ap)
    add_tracking_arguments(ap)
    return ap


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    ns = parser.parse_args(argv)
    video: Path = ns.video
    out_dir: Path = ns.out_dir
    try:
        detect, track, stability = build_detect_config(ns), build_track_config(ns), build_stability_config(ns)
        match = MatchConfig(radius=ns.match_radius)
        amplitudes: list[float] = ns.amplitudes
        sigmas: list[float] = ns.sigmas
        motions: list[Motion] = ns.motions
        classes = bench_classes(motions, amplitudes, sigmas)
        sampling = Sampling(ns.seed, classes, ns.per_class, 3 * match.radius)
    except ValueError as err:
        parser.error(str(err))
    if shutil.which("ffprobe") is None:
        parser.error("ffprobe not found in PATH")
    dest = out_dir / video.stem
    try:
        setup = BenchSetup(video, detect, track, stability, sampling, ns.workers)
        run = load_or_collect(setup, dest / "cache.pkl")
    except (VideoError, ValueError, OSError) as err:
        print(f"{video}: {err}", file=sys.stderr)
        return 1
    evaluation = evaluate(run, track, match, detect.osd_regions, stability)
    rows = summarize(evaluation)
    report = {
        "video": str(video),
        "code": code_version(),
        "detect": asdict(detect),
        "track": asdict(track),
        "stability": asdict(stability),
        "match": asdict(match),
        "sampling": asdict(sampling),
        "summary": rows,
        "not_visible": evaluation.not_visible,
        "false_tracks": evaluation.false_tracks,
        "reference_tracks": evaluation.reference_tracks,
        "injected_tracks": evaluation.injected_tracks,
        "bats": [asdict(b) for b in evaluation.bats],
    }
    (dest / "bench.json").write_text(json.dumps(report, indent=2) + "\n")
    print(format_table(rows, evaluation))
    return 0


if __name__ == "__main__":
    sys.exit(main())
