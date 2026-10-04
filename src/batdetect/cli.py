from __future__ import annotations

import argparse
import shutil
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

from batdetect.output import (
    RenderConfig,
    config_params,
    format_time,
    render_annotated,
    split_clips,
    summary_image,
    write_params,
    write_tracks_csv,
)
from batdetect.parallel import default_workers, detect_video
from batdetect.pipeline import (
    DetectConfig,
    Region,
    TrackConfig,
    VideoError,
    open_video,
    track_detections,
)

VIDEO_EXTENSIONS = frozenset({".mp4", ".mov", ".avi", ".mkv", ".m4v"})


@dataclass(frozen=True, slots=True)
class Job:
    video: Path
    dest: Path


def collect_videos(inputs: list[Path]) -> list[Path]:
    videos: list[Path] = []
    for path in inputs:
        if path.is_dir():
            videos.extend(sorted(f for f in path.iterdir() if f.is_file() and f.suffix.lower() in VIDEO_EXTENSIONS))
        else:
            videos.append(path)
    return videos


def output_name(video: Path, ambiguous: bool) -> str:
    if not ambiguous:
        return video.stem
    return f"{video.parent.name}_{video.stem}_{video.suffix.lstrip('.').lower()}"


def plan_jobs(videos: list[Path], out_dir: Path) -> list[Job]:
    stems = Counter(v.stem for v in videos)
    jobs = [Job(v, out_dir / output_name(v, stems[v.stem] > 1)) for v in videos]
    clashes = [dest for dest, n in Counter(j.dest for j in jobs).items() if n > 1]
    if clashes:
        raise ValueError(f"several videos would write to {clashes[0]}")
    return jobs


def parse_region(text: str) -> Region:
    try:
        x0, y0, x1, y1 = (float(v) for v in text.split(","))
        return Region(x0, y0, x1, y1)
    except ValueError as err:
        raise argparse.ArgumentTypeError(f"expected x0,y0,x1,y1 fractions, got {text!r}: {err}") from err


def add_detection_arguments(ap: argparse.ArgumentParser) -> None:
    d = DetectConfig()
    detect = ap.add_argument_group("detection")
    detect.add_argument("--threshold", type=float, default=d.threshold)
    detect.add_argument("--min-area", type=float, default=d.min_area, help="source pixels²")
    detect.add_argument("--max-area", type=float, default=d.max_area, help="source pixels²")
    detect.add_argument("--bg-window", type=float, default=d.bg_window_s, help="seconds of rolling median background")
    detect.add_argument("--bg-step", type=int, default=d.bg_step)
    detect.add_argument(
        "--osd-region",
        type=parse_region,
        action="append",
        default=[],
        help="x0,y0,x1,y1 as fractions of the frame; masks a moving on-screen display",
    )
    detect.add_argument(
        "--merge-radius", type=float, default=d.merge_radius, help="source pixels bridged between fragments"
    )
    detect.add_argument("--work-width", type=int, default=d.work_width)


def add_tracking_arguments(ap: argparse.ArgumentParser) -> None:
    t = TrackConfig()
    track = ap.add_argument_group("tracking")
    track.add_argument("--max-jump", type=float, default=t.max_jump, help="source pixels")
    track.add_argument("--max-gap", type=int, default=t.max_gap)
    track.add_argument("--min-hits", type=int, default=t.min_hits)
    track.add_argument("--min-travel", type=float, default=t.min_travel, help="source pixels")
    track.add_argument(
        "--twin-distance", type=float, default=t.twin_distance, help="source pixels between fragments of one animal"
    )


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


def build_detect_config(ns: argparse.Namespace) -> DetectConfig:
    return DetectConfig(
        threshold=ns.threshold,
        min_area=ns.min_area,
        max_area=ns.max_area,
        bg_window_s=ns.bg_window,
        bg_step=ns.bg_step,
        osd_regions=tuple(ns.osd_region),
        merge_radius=ns.merge_radius,
        work_width=ns.work_width,
    )


def build_track_config(ns: argparse.Namespace) -> TrackConfig:
    return TrackConfig(
        max_jump=ns.max_jump,
        max_gap=ns.max_gap,
        min_hits=ns.min_hits,
        min_travel=ns.min_travel,
        twin_distance=ns.twin_distance,
    )


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
