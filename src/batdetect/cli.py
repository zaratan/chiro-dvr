from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path

from batdetect.arguments import add_detection_arguments, add_tracking_arguments, build_detect_config, build_track_config
from batdetect.detect import DetectConfig
from batdetect.jobs import Job, collect_videos, plan_jobs
from batdetect.output.annotated import render_annotated
from batdetect.output.clips import split_clips
from batdetect.output.config import RenderConfig
from batdetect.output.summary import summary_image
from batdetect.output.tables import config_params, write_params, write_tracks_csv
from batdetect.output.timefmt import format_time
from batdetect.parallel import default_workers, detect_video
from batdetect.track import TrackConfig, track_detections
from batdetect.video import VideoError, open_video


def build_parser() -> argparse.ArgumentParser:
    r = RenderConfig()
    ap = argparse.ArgumentParser(prog="batdetect", description="Detect and track bats in thermal videos.")
    ap.add_argument("inputs", nargs="+", type=Path, help="video files or folders")
    ap.add_argument("-o", "--out-dir", type=Path, default=Path("out"))
    ap.add_argument("--workers", type=int, default=default_workers(), help="parallel processes for detection")
    add_detection_arguments(ap)
    add_tracking_arguments(ap)
    render = ap.add_argument_group("output")
    render.add_argument("--box-pad", type=int, default=r.box_pad)
    render.add_argument("--trail", type=float, default=r.trail_s)
    render.add_argument("--clip-margin", type=float, default=r.clip_margin_s)
    render.add_argument("--crf", type=int, default=r.crf)
    return ap


def build_configs(ns: argparse.Namespace) -> tuple[DetectConfig, TrackConfig, RenderConfig]:
    render = RenderConfig(box_pad=ns.box_pad, trail_s=ns.trail, clip_margin_s=ns.clip_margin, crf=ns.crf)
    return build_detect_config(ns), build_track_config(ns), render


def process(job: Job, detect: DetectConfig, track: TrackConfig, render: RenderConfig, workers: int) -> None:
    cap, info = open_video(job.video, detect.work_width)
    cap.release()
    detections = detect_video(job.video, detect, info, workers)[0]
    tracks = track_detections(detections, track)
    job.dest.mkdir(parents=True, exist_ok=True)
    stem = job.video.stem
    write_tracks_csv(tracks, info, job.dest / f"{stem}.tracks.csv")
    write_params(
        {"video": str(job.video), "fps": info.fps, **config_params(detect, track, render)},
        job.dest / "params.json",
    )
    summary_image(job.video, tracks, info, job.dest / f"{stem}.tracks.png")
    annotated = job.dest / f"{stem}_boxes.mp4"
    render_annotated(job.video, tracks, info, annotated, render)
    split_clips(annotated, tracks, info, job.dest / "split", render)
    print(f"{job.video.name}: {len(detections)} frames, {len(tracks)} tracks -> {job.dest}")
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
            process(job, *configs, max(1, ns.workers))
        except (VideoError, ValueError, OSError) as err:
            failures += 1
            print(f"{job.video}: {err}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
