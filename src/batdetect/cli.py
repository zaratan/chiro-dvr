from __future__ import annotations

import argparse
import multiprocessing
import shutil
import sys
from dataclasses import dataclass, replace
from importlib.metadata import version
from pathlib import Path

from batdetect.arguments import (
    add_detection_arguments,
    add_tracking_arguments,
    build_detect_config,
    build_stability_config,
    build_track_config,
    positive_int,
)
from batdetect.detect import DetectConfig
from batdetect.exclusion import exclude
from batdetect.jobs import Job, collect_videos, plan_jobs
from batdetect.output.background import hide_display, median_background
from batdetect.output.clips import clip_windows
from batdetect.output.config import ENCODERS, MAX_CRF, MAX_VT_QUALITY, ZOOMS, RenderConfig
from batdetect.output.encoder import resolve_encoder
from batdetect.output.overlay import track_overlay
from batdetect.output.periods import split_by_period
from batdetect.output.render import VideoOutputs, discard_videos, render_videos
from batdetect.output.summary import summary_image
from batdetect.output.tables import config_params, write_params, write_tracks_csv
from batdetect.output.timefmt import format_time
from batdetect.output.zoomview import zoom_windows
from batdetect.parallel import DEFAULT_WORKERS, detect_video
from batdetect.probe import ffprobe_version, probing
from batdetect.spans import Span, covered_frames
from batdetect.stability import StabilityConfig
from batdetect.track import TrackConfig, track_detections
from batdetect.video import VideoError, open_video


class TooManyTracksError(Exception):
    pass


@dataclass(frozen=True, slots=True)
class Settings:
    detect: DetectConfig
    track: TrackConfig
    stability: StabilityConfig
    render: RenderConfig
    mode: str


def build_parser() -> argparse.ArgumentParser:
    r = RenderConfig()
    ap = argparse.ArgumentParser(prog="batdetect", description="Detect and track bats in thermal videos.")
    ap.add_argument("--version", action="version", version=f"batdetect {version('batdetect')}")
    ap.add_argument("inputs", nargs="+", type=Path, help="video files or folders")
    ap.add_argument(
        "-o",
        "--out-dir",
        type=Path,
        default=Path("out"),
        metavar="DIR",
        help="folder that receives one result folder per video (default: %(default)s)",
    )
    ap.add_argument(
        "--workers",
        type=positive_int,
        default=DEFAULT_WORKERS,
        metavar="COUNT",
        help="processes sharing the detection; more mostly adds re-reading of the video (default: %(default)s)",
    )
    add_detection_arguments(ap)
    add_tracking_arguments(ap)
    render = ap.add_argument_group("output")
    render.add_argument(
        "--box-pad",
        type=int,
        default=r.box_pad,
        metavar="PIXELS",
        help="space between a target and its box on the clips; source pixels (default: %(default)s)",
    )
    render.add_argument(
        "--trail",
        type=float,
        default=r.trail_s,
        metavar="SECONDS",
        help="length of the yellow trail behind each target on the clips; seconds (default: %(default)s)",
    )
    render.add_argument(
        "--clip-margin",
        type=float,
        default=r.clip_margin_s,
        metavar="SECONDS",
        help="time kept before and after each track in its clip; seconds (default: %(default)s)",
    )
    render.add_argument(
        "--crf",
        type=int,
        default=r.crf,
        metavar="QUALITY",
        help=f"video quality with x264, lower is better and heavier; 0 to {MAX_CRF} (default: %(default)s)",
    )
    render.add_argument(
        "--encoder",
        choices=ENCODERS,
        default=r.encoder,
        help="auto uses the Apple media engine if usable, otherwise x264 (default: %(default)s)",
    )
    render.add_argument(
        "--vt-quality",
        type=int,
        default=r.vt_quality,
        metavar="QUALITY",
        help="video quality with the Apple media engine, higher is better and heavier; "
        f"1 to {MAX_VT_QUALITY} (default: %(default)s)",
    )
    render.add_argument(
        "--annotated",
        action="store_true",
        help="also write the whole video with every track drawn (default: off)",
    )
    render.add_argument(
        "--max-tracks",
        type=int,
        default=r.max_tracks,
        metavar="COUNT",
        help="above this many tracks, the video counts as failed and gets no clips or annotated video; "
        "0 disables (default: %(default)s)",
    )
    render.add_argument(
        "--zoom",
        choices=ZOOMS,
        default=r.zoom,
        help="slowed, magnified clip: auto for small or faint tracks only, all, or none (default: %(default)s)",
    )
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
        max_tracks=ns.max_tracks,
        zoom=ns.zoom,
    )
    return Settings(build_detect_config(ns), build_track_config(ns), build_stability_config(ns), render, ns.mode)


def seconds(span: Span, fps: float) -> tuple[float, float]:
    return span.first / fps, (span.last + 1) / fps


def process(job: Job, settings: Settings, workers: int, ffprobe: str) -> None:
    detect, render = settings.detect, settings.render
    cap, info = open_video(job.video, detect.work_width)
    cap.release()
    with probing(job.video) as outcome:
        detections = detect_video(job.video, detect, info, workers)[0]
    analysis = exclude(detections, outcome.probe, detect.half_window(info.fps), info.fps, settings.stability)
    unstable = analysis.unstable
    tracks = track_detections(analysis.detections, settings.track)
    job.dest.mkdir(parents=True, exist_ok=True)
    stem = job.video.stem
    write_tracks_csv(tracks, info, job.dest / f"{stem}.tracks.csv")
    write_params(
        {
            "video": str(job.video),
            "fps": info.fps,
            "mode": settings.mode,
            **config_params(detect, settings.track, settings.stability, render),
            "ignored_s": [[round(t, 2) for t in seconds(span, info.fps)] for span in unstable],
            "damaged_s": [[round(t, 2) for t in seconds(span, info.fps)] for span in analysis.damaged],
            "damaged_frames": analysis.reported_frames,
            "ffprobe": ffprobe,
        },
        job.dest / "params.json",
    )
    periods = split_by_period(
        tracks,
        info,
        ignored=[seconds(span, info.fps) for span in unstable],
        damaged=[seconds(span, info.fps) for span in analysis.damaged],
    )
    summary_image(hide_display(median_background(job.video, info), detect), periods, info, job.dest / stem)
    split_dir = job.dest / "split"
    clips = clip_windows(tracks, info.fps, render.clip_margin_s, split_dir)
    outputs = VideoOutputs(split_dir, clips + zoom_windows(tracks, clips, info, render), job.dest / f"{stem}_boxes.mp4")
    renders_videos = render.renders_videos_for(len(tracks))
    if renders_videos:
        render_videos(job.video, info, outputs, track_overlay(tracks, info, render), render)
    else:
        discard_videos(outputs)
    ignored = len(analysis.ignored_frames) / info.fps
    print(f"{job.video.name}: {len(detections)} frames, {len(tracks)} tracks, {ignored:.1f} s ignored -> {job.dest}")
    if analysis.damaged:
        damaged = len(covered_frames(analysis.damaged)) / info.fps
        count = len(analysis.damaged)
        print(f"  damaged {damaged:.1f} s in {count} span{'s' if count > 1 else ''}, detections ignored")
    for span in unstable:
        start, end = (format_time(t) for t in seconds(span, info.fps))
        print(f"  unstable {start} -> {end}, detections ignored")
    if renders_videos:
        for t in tracks:
            start, end = format_time(t.first.frame / info.fps), format_time(t.last.frame / info.fps)
            print(f"  #{t.id:<3} {start} -> {end}  hits={len(t.points)}")
    else:
        raise TooManyTracksError(
            f"{len(tracks)} tracks, above --max-tracks {render.max_tracks}: clips and annotated video skipped"
        )


def main(argv: list[str] | None = None) -> int:
    multiprocessing.freeze_support()
    parser = build_parser()
    ns = parser.parse_args(argv)
    try:
        configs = build_configs(ns)
    except ValueError as err:
        parser.error(str(err))
    if shutil.which("ffmpeg") is None:
        parser.error("ffmpeg not found in PATH")
    if shutil.which("ffprobe") is None:
        parser.error("ffprobe not found in PATH")
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
    ffprobe = ffprobe_version()
    failures = 0
    for job in jobs:
        try:
            process(job, settings, ns.workers, ffprobe)
        except (VideoError, ValueError, OSError, TooManyTracksError) as err:
            failures += 1
            print(f"{job.video}: {err}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
