from __future__ import annotations

import argparse
import shutil
import sys
from dataclasses import dataclass, replace
from pathlib import Path

from batdetect.arguments import (
    add_detection_arguments,
    add_tracking_arguments,
    build_detect_config,
    build_stability_config,
    build_track_config,
)
from batdetect.detect import DetectConfig
from batdetect.jobs import Job, collect_videos, plan_jobs
from batdetect.output.background import hide_display, median_background
from batdetect.output.clips import clip_windows
from batdetect.output.config import ENCODERS, RenderConfig
from batdetect.output.encoder import resolve_encoder
from batdetect.output.overlay import track_overlay
from batdetect.output.periods import split_by_period
from batdetect.output.render import VideoOutputs, render_videos
from batdetect.output.summary import summary_image
from batdetect.output.tables import config_params, write_params, write_tracks_csv
from batdetect.output.timefmt import format_time
from batdetect.parallel import DEFAULT_WORKERS, detect_video
from batdetect.stability import Span, StabilityConfig, unstable_spans, without_spans
from batdetect.track import TrackConfig, track_detections
from batdetect.video import VideoError, open_video


@dataclass(frozen=True, slots=True)
class Settings:
    detect: DetectConfig
    track: TrackConfig
    stability: StabilityConfig
    render: RenderConfig


def build_parser() -> argparse.ArgumentParser:
    r = RenderConfig()
    ap = argparse.ArgumentParser(prog="batdetect", description="Detect and track bats in thermal videos.")
    ap.add_argument("inputs", nargs="+", type=Path, help="video files or folders")
    ap.add_argument("-o", "--out-dir", type=Path, default=Path("out"))
    ap.add_argument("--workers", type=int, default=DEFAULT_WORKERS, help="parallel processes for detection")
    add_detection_arguments(ap)
    add_tracking_arguments(ap)
    render = ap.add_argument_group("output")
    render.add_argument("--box-pad", type=int, default=r.box_pad)
    render.add_argument("--trail", type=float, default=r.trail_s)
    render.add_argument("--clip-margin", type=float, default=r.clip_margin_s)
    render.add_argument("--crf", type=int, default=r.crf, help="quality for x264 (lower is better)")
    render.add_argument(
        "--encoder", choices=ENCODERS, default=r.encoder, help="auto uses the Apple media engine if usable"
    )
    render.add_argument(
        "--vt-quality", type=int, default=r.vt_quality, help="quality for videotoolbox (higher is better)"
    )
    render.add_argument("--annotated", action="store_true", help="also write the whole annotated video")
    return ap


def build_configs(ns: argparse.Namespace) -> Settings:
    render = RenderConfig(
        box_pad=ns.box_pad,
        trail_s=ns.trail,
        clip_margin_s=ns.clip_margin,
        crf=ns.crf,
        encoder=ns.encoder,
        vt_quality=ns.vt_quality,
        annotated=ns.annotated,
    )
    return Settings(build_detect_config(ns), build_track_config(ns), build_stability_config(ns), render)


def seconds(span: Span, fps: float) -> tuple[float, float]:
    return span.first / fps, (span.last + 1) / fps


def process(job: Job, settings: Settings, workers: int) -> None:
    detect, render = settings.detect, settings.render
    cap, info = open_video(job.video, detect.work_width)
    cap.release()
    detections = detect_video(job.video, detect, info, workers)[0]
    spans = unstable_spans(detections, info.fps, settings.stability)
    tracks = track_detections(without_spans(detections, spans), settings.track)
    job.dest.mkdir(parents=True, exist_ok=True)
    stem = job.video.stem
    write_tracks_csv(tracks, info, job.dest / f"{stem}.tracks.csv")
    write_params(
        {
            "video": str(job.video),
            "fps": info.fps,
            **config_params(detect, settings.track, settings.stability, render),
            "ignored_s": [[round(t, 2) for t in seconds(span, info.fps)] for span in spans],
        },
        job.dest / "params.json",
    )
    periods = split_by_period(tracks, info, ignored=[seconds(span, info.fps) for span in spans])
    summary_image(hide_display(median_background(job.video, info), detect), periods, info, job.dest / stem)
    split_dir = job.dest / "split"
    outputs = VideoOutputs(
        split_dir, clip_windows(tracks, info.fps, render.clip_margin_s, split_dir), job.dest / f"{stem}_boxes.mp4"
    )
    render_videos(job.video, info, outputs, track_overlay(tracks, info, render), render)
    ignored = sum(end - start for start, end in (seconds(span, info.fps) for span in spans))
    print(f"{job.video.name}: {len(detections)} frames, {len(tracks)} tracks, {ignored:.1f} s ignored -> {job.dest}")
    for span in spans:
        start, end = (format_time(t) for t in seconds(span, info.fps))
        print(f"  unstable {start} -> {end}, detections ignored")
    for t in tracks:
        start, end = format_time(t.first.frame / info.fps), format_time(t.last.frame / info.fps)
        print(f"  #{t.id:<3} {start} -> {end}  hits={len(t.points)}")


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    ns = parser.parse_args(argv)
    try:
        configs = build_configs(ns)
    except ValueError as err:
        parser.error(str(err))
    if shutil.which("ffmpeg") is None:
        parser.error("ffmpeg not found in PATH")
    try:
        settings = replace(configs, render=resolve_encoder(configs.render))
    except ValueError as err:
        parser.error(str(err))
    out_dir: Path = ns.out_dir
    inputs: list[Path] = ns.inputs
    try:
        jobs = plan_jobs(collect_videos(inputs), out_dir)
    except ValueError as err:
        parser.error(str(err))
    if not jobs:
        parser.error("no video found")
    failures = 0
    for job in jobs:
        try:
            process(job, settings, max(1, ns.workers))
        except (VideoError, ValueError, OSError) as err:
            failures += 1
            print(f"{job.video}: {err}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
