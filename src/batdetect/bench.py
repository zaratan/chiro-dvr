from __future__ import annotations

import argparse
import hashlib
import json
import math
import pickle
import statistics
import subprocess
import sys
from collections.abc import Iterator, Sequence
from dataclasses import asdict, dataclass, field
from pathlib import Path

import cv2

from batdetect.cli import add_detection_arguments, add_tracking_arguments, build_detect_config, build_track_config
from batdetect.pipeline import (
    SYMBION_OSD,
    DetectConfig,
    Detection,
    GrayFrame,
    Track,
    TrackConfig,
    VideoError,
    VideoInfo,
    detect_frames,
    open_video,
    read_frames,
    read_gray_frames,
    to_work_gray,
    track_detections,
)
from batdetect.synthetic import BatClass, Sampling, SyntheticBat, inject, random_bats, reference_occupancy

VISIBLE_MIN_CONTRAST = 8.0
VISIBLE_MARGIN_SIGMAS = 2.0
WILSON_Z = 1.96
DEFAULT_AMPLITUDES = (-15.0, -30.0, -60.0)
DEFAULT_SIGMAS = (1.5, 3.0, 5.0)


@dataclass(frozen=True, slots=True)
class Observation:
    frame: int
    x: float
    y: float
    effective: float


@dataclass(frozen=True, slots=True)
class MatchConfig:
    radius: float = 8.0
    purity: float = 0.5


@dataclass(slots=True)
class BenchRun:
    key: str
    info: VideoInfo
    reference: dict[int, list[Detection]]
    injected: dict[int, list[Detection]]
    bats: list[SyntheticBat]
    observations: dict[int, list[Observation]] = field(default_factory=dict[int, list[Observation]])


@dataclass(frozen=True, slots=True)
class BatResult:
    id: int
    amplitude: float
    sigma: float
    visible_frames: int
    found: bool
    completeness: float
    start_delay: int | None
    end_early: int | None
    fragments: int
    median_effective: float


@dataclass(frozen=True, slots=True)
class Evaluation:
    bats: list[BatResult]
    not_visible: int
    false_tracks: int
    reference_tracks: int
    injected_tracks: int


def is_visible(obs: Observation, sigma: float, info: VideoInfo) -> bool:
    margin = VISIBLE_MARGIN_SIGMAS * sigma
    if not (margin <= obs.x <= info.width - margin and margin <= obs.y <= info.height - margin):
        return False
    rel_x, rel_y = obs.x / info.width, obs.y / info.height
    if any(region.contains(rel_x, rel_y) for region in SYMBION_OSD):
        return False
    return abs(obs.effective) >= VISIBLE_MIN_CONTRAST


def cache_key(video: Path, detect: DetectConfig, sampling: Sampling) -> str:
    stat = video.stat()
    payload = {
        "video": str(video.resolve()),
        "size": stat.st_size,
        "mtime": stat.st_mtime,
        "detect": asdict(detect),
        "sampling": asdict(sampling),
    }
    return hashlib.sha256(json.dumps(payload, sort_keys=True).encode()).hexdigest()


def _injected_frames(
    video: Path, info: VideoInfo, bats: Sequence[SyntheticBat], observations: dict[int, list[Observation]]
) -> Iterator[GrayFrame]:
    active: dict[int, list[SyntheticBat]] = {}
    for bat in bats:
        for frame in range(bat.start_frame, bat.end_frame + 1):
            active.setdefault(frame, []).append(bat)
    cap = cv2.VideoCapture(str(video))
    if not cap.isOpened():
        raise VideoError(f"cannot open {video}")
    try:
        for frame_no, frame in enumerate(read_frames(cap)):
            here = active.get(frame_no, [])
            effective = inject(frame, here, frame_no)
            for bat in here:
                pos = bat.position(frame_no)
                if pos is not None and bat.id in effective:
                    observations.setdefault(bat.id, []).append(Observation(frame_no, *pos, effective[bat.id]))
            yield to_work_gray(frame, info)
    finally:
        cap.release()


def collect(video: Path, detect: DetectConfig, track: TrackConfig, sampling: Sampling) -> BenchRun:
    cap, info = open_video(video, detect.work_width)
    try:
        reference = detect_frames(read_gray_frames(cap, info), info.fps, detect)
    finally:
        cap.release()
    occupied = reference_occupancy(track_detections(reference, track), info.scale)
    bats = random_bats(sampling, info, len(reference), occupied)
    observations: dict[int, list[Observation]] = {}
    injected = detect_frames(_injected_frames(video, info, bats, observations), info.fps, detect)
    return BenchRun(cache_key(video, detect, sampling), info, reference, injected, bats, observations)


def load_or_collect(video: Path, detect: DetectConfig, track: TrackConfig, sampling: Sampling, cache: Path) -> BenchRun:
    key = cache_key(video, detect, sampling)
    if cache.exists():
        with cache.open("rb") as fh:
            cached: object = pickle.load(fh)
        if isinstance(cached, BenchRun) and cached.key == key:
            return cached
    run = collect(video, detect, track, sampling)
    cache.parent.mkdir(parents=True, exist_ok=True)
    tmp = cache.with_suffix(".tmp")
    with tmp.open("wb") as fh:
        pickle.dump(run, fh)
    tmp.replace(cache)
    return run


def _near(det: Detection, x: float, y: float, radius: float) -> bool:
    return math.hypot(det.x - x, det.y - y) <= radius


def _truth_by_frame(run: BenchRun) -> dict[int, list[tuple[int, float, float]]]:
    truth: dict[int, list[tuple[int, float, float]]] = {}
    for bat_id, observations in run.observations.items():
        for obs in observations:
            truth.setdefault(obs.frame, []).append((bat_id, obs.x / run.info.scale, obs.y / run.info.scale))
    return truth


def assign_tracks(
    tracks: Sequence[Track], truth: dict[int, list[tuple[int, float, float]]], match: MatchConfig
) -> dict[int, list[Track]]:
    assigned: dict[int, list[Track]] = {}
    for track in tracks:
        counts: dict[int, int] = {}
        for det in track.points:
            for bat_id, x, y in truth.get(det.frame, []):
                if _near(det, x, y, match.radius):
                    counts[bat_id] = counts.get(bat_id, 0) + 1
        if not counts:
            continue
        best = max(counts, key=lambda b: counts[b])
        if counts[best] / len(track.points) >= match.purity:
            assigned.setdefault(best, []).append(track)
    return assigned


def _matches_reference(track: Track, reference: dict[int, list[Detection]], match: MatchConfig) -> bool:
    near = sum(any(_near(det, r.x, r.y, match.radius) for r in reference.get(det.frame, [])) for det in track.points)
    return near / len(track.points) >= match.purity


def _reference_points(tracks: Sequence[Track]) -> dict[int, list[Detection]]:
    points: dict[int, list[Detection]] = {}
    for track in tracks:
        for det in track.points:
            points.setdefault(det.frame, []).append(det)
    return points


def evaluate(run: BenchRun, track: TrackConfig, match: MatchConfig) -> Evaluation:
    truth = _truth_by_frame(run)
    injected_tracks = track_detections(run.injected, track)
    reference_tracks = track_detections(run.reference, track)
    assigned = assign_tracks(injected_tracks, truth, match)
    results: list[BatResult] = []
    not_visible = 0
    for bat in run.bats:
        observations = run.observations.get(bat.id, [])
        visible = {o.frame: o for o in observations if is_visible(o, bat.sigma, run.info)}
        if len(visible) < track.min_hits:
            not_visible += 1
            continue
        covered = {
            det.frame
            for t in assigned.get(bat.id, [])
            for det in t.points
            if det.frame in visible
            and _near(det, visible[det.frame].x / run.info.scale, visible[det.frame].y / run.info.scale, match.radius)
        }
        found = len(covered) >= track.min_hits
        results.append(
            BatResult(
                id=bat.id,
                amplitude=bat.amplitude,
                sigma=bat.sigma,
                visible_frames=len(visible),
                found=found,
                completeness=len(covered) / len(visible),
                start_delay=min(covered) - min(visible) if found else None,
                end_early=max(visible) - max(covered) if found else None,
                fragments=len(assigned.get(bat.id, [])),
                median_effective=statistics.median(o.effective for o in visible.values()),
            )
        )
    assigned_ids = {id(t) for tracks in assigned.values() for t in tracks}
    reference_points = _reference_points(reference_tracks)
    false_tracks = sum(
        1 for t in injected_tracks if id(t) not in assigned_ids and not _matches_reference(t, reference_points, match)
    )
    return Evaluation(results, not_visible, false_tracks, len(reference_tracks), len(injected_tracks))


def wilson(successes: int, total: int) -> tuple[float, float]:
    if total == 0:
        return 0.0, 0.0
    p = successes / total
    denom = 1 + WILSON_Z**2 / total
    center = (p + WILSON_Z**2 / (2 * total)) / denom
    half = WILSON_Z * math.sqrt(p * (1 - p) / total + WILSON_Z**2 / (4 * total**2)) / denom
    return max(0.0, center - half), min(1.0, center + half)


def _median_or_none(values: Sequence[float]) -> float | None:
    return statistics.median(values) if values else None


def summarize(evaluation: Evaluation) -> list[dict[str, object]]:
    groups: dict[tuple[float, float], list[BatResult]] = {}
    for result in evaluation.bats:
        groups.setdefault((result.amplitude, result.sigma), []).append(result)
    rows: list[dict[str, object]] = []
    for (amplitude, sigma), results in sorted(groups.items()):
        found = [r for r in results if r.found]
        low, high = wilson(len(found), len(results))
        rows.append(
            {
                "amplitude": amplitude,
                "sigma": sigma,
                "n": len(results),
                "found": len(found),
                "found_ci95": [round(low, 3), round(high, 3)],
                "median_completeness": round(statistics.median(r.completeness for r in results), 3),
                "median_start_delay": _median_or_none([r.start_delay for r in found if r.start_delay is not None]),
                "median_end_early": _median_or_none([r.end_early for r in found if r.end_early is not None]),
                "mean_fragments": round(statistics.mean(r.fragments for r in found), 2) if found else None,
                "median_effective": round(statistics.median(r.median_effective for r in results), 1),
            }
        )
    return rows


def code_version() -> str:
    result = subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True, check=False)
    return result.stdout.strip() or "unknown"


def format_table(rows: Sequence[dict[str, object]], evaluation: Evaluation) -> str:
    lines = ["amplitude  sigma   n  found  ci95          completeness  start_delay  end_early  fragments  effective"]
    for r in rows:
        lines.append(
            f"{r['amplitude']:>9}  {r['sigma']:>5}  {r['n']:>2}  {r['found']:>5}  {r['found_ci95']!s:<12}  "
            f"{r['median_completeness']!s:>12}  {r['median_start_delay']!s:>11}  {r['median_end_early']!s:>9}  "
            f"{r['mean_fragments']!s:>9}  {r['median_effective']!s:>9}"
        )
    lines.append(
        f"not visible: {evaluation.not_visible}  false tracks: {evaluation.false_tracks}  "
        f"tracks: {evaluation.injected_tracks} injected run, {evaluation.reference_tracks} reference"
    )
    return "\n".join(lines)


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(prog="batdetect-bench", description="Measure detection on injected synthetic bats.")
    ap.add_argument("video", type=Path)
    ap.add_argument("-o", "--out-dir", type=Path, default=Path("out/bench"))
    ap.add_argument("--per-class", type=int, default=30)
    ap.add_argument("--amplitudes", type=float, nargs="+", default=list(DEFAULT_AMPLITUDES))
    ap.add_argument("--sigmas", type=float, nargs="+", default=list(DEFAULT_SIGMAS), help="blob size, source pixels")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--match-radius", type=float, default=MatchConfig().radius, help="work pixels")
    add_detection_arguments(ap)
    add_tracking_arguments(ap)
    return ap


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    ns = parser.parse_args(argv)
    video: Path = ns.video
    out_dir: Path = ns.out_dir
    try:
        detect, track = build_detect_config(ns), build_track_config(ns)
        match = MatchConfig(radius=ns.match_radius)
        amplitudes: list[float] = ns.amplitudes
        sigmas: list[float] = ns.sigmas
        classes = tuple(BatClass(a, s) for a in amplitudes for s in sigmas)
        sampling = Sampling(ns.seed, classes, ns.per_class, 3 * match.radius)
    except ValueError as err:
        parser.error(str(err))
    dest = out_dir / video.stem
    try:
        run = load_or_collect(video, detect, track, sampling, dest / "cache.pkl")
    except VideoError as err:
        print(f"{video}: {err}", file=sys.stderr)
        return 1
    evaluation = evaluate(run, track, match)
    rows = summarize(evaluation)
    report = {
        "video": str(video),
        "code": code_version(),
        "detect": asdict(detect),
        "track": asdict(track),
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
