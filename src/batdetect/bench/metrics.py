from __future__ import annotations

import statistics
from collections.abc import Sequence
from dataclasses import dataclass

from batdetect.bench.collect import BenchRun
from batdetect.bench.config import MatchConfig
from batdetect.bench.matching import assign_tracks, matches_reference, near, reference_points, truth_by_frame
from batdetect.detect import Region
from batdetect.exclusion import exclude
from batdetect.spans import covered_frames
from batdetect.stability import StabilityConfig
from batdetect.synthetic.injection import Observation
from batdetect.synthetic.trajectory import Motion
from batdetect.track import TrackConfig, track_detections
from batdetect.video import VideoInfo

VISIBLE_MIN_CONTRAST = 8.0


VISIBLE_MARGIN_SIGMAS = 2.0


DEFAULT_STABILITY = StabilityConfig()


@dataclass(frozen=True, slots=True)
class BatResult:
    id: int
    motion: Motion
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


def is_visible(obs: Observation, sigma: float, info: VideoInfo, masked: Sequence[Region] = ()) -> bool:
    margin = VISIBLE_MARGIN_SIGMAS * sigma
    if not (margin <= obs.x <= info.width - margin and margin <= obs.y <= info.height - margin):
        return False
    if any(region.contains(obs.x / info.width, obs.y / info.height) for region in masked):
        return False
    return abs(obs.effective) >= VISIBLE_MIN_CONTRAST


def evaluate(
    run: BenchRun,
    track: TrackConfig,
    match: MatchConfig,
    masked: Sequence[Region] = (),
    stability: StabilityConfig = DEFAULT_STABILITY,
) -> Evaluation:
    truth = truth_by_frame(run)
    injected = exclude(run.injected, run.probe, run.margin, run.info.fps, stability)
    ignored = covered_frames(injected.ignored_spans)
    injected_tracks = track_detections(injected.detections, track)
    reference_tracks = track_detections(
        exclude(run.reference, run.probe, run.margin, run.info.fps, stability).detections, track
    )
    assigned = assign_tracks(injected_tracks, truth, match)
    results: list[BatResult] = []
    not_visible = 0
    for bat in run.bats:
        observations = run.observations.get(bat.id, [])
        visible = {
            o.frame: o for o in observations if is_visible(o, bat.sigma, run.info, masked) and o.frame not in ignored
        }
        if len(visible) < track.min_hits:
            not_visible += 1
            continue
        covered = {
            det.frame
            for t in assigned.get(bat.id, [])
            for det in t.points
            if det.frame in visible and near(det, visible[det.frame].x, visible[det.frame].y, match.radius)
        }
        found = len(covered) >= track.min_hits
        results.append(
            BatResult(
                id=bat.id,
                motion=bat.motion,
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
    points_by_frame = reference_points(reference_tracks)
    false_tracks = sum(
        1 for t in injected_tracks if id(t) not in assigned_ids and not matches_reference(t, points_by_frame, match)
    )
    return Evaluation(results, not_visible, false_tracks, len(reference_tracks), len(injected_tracks))
