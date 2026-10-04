from __future__ import annotations

import math
from collections.abc import Sequence

from batdetect.bench.collect import BenchRun
from batdetect.bench.config import MatchConfig
from batdetect.detect import Detection
from batdetect.track import Track


def near(det: Detection, x: float, y: float, radius: float) -> bool:
    return math.hypot(det.x - x, det.y - y) <= radius


def truth_by_frame(run: BenchRun) -> dict[int, list[tuple[int, float, float]]]:
    truth: dict[int, list[tuple[int, float, float]]] = {}
    for bat_id, observations in run.observations.items():
        for obs in observations:
            truth.setdefault(obs.frame, []).append((bat_id, obs.x, obs.y))
    return truth


def assign_tracks(
    tracks: Sequence[Track], truth: dict[int, list[tuple[int, float, float]]], match: MatchConfig
) -> dict[int, list[Track]]:
    assigned: dict[int, list[Track]] = {}
    for track in tracks:
        counts: dict[int, int] = {}
        for det in track.points:
            for bat_id, x, y in truth.get(det.frame, []):
                if near(det, x, y, match.radius):
                    counts[bat_id] = counts.get(bat_id, 0) + 1
        if not counts:
            continue
        best = max(counts, key=lambda b: counts[b])
        if counts[best] / len(track.points) >= match.purity:
            assigned.setdefault(best, []).append(track)
    return assigned


def matches_reference(track: Track, reference: dict[int, list[Detection]], match: MatchConfig) -> bool:
    close = sum(any(near(det, r.x, r.y, match.radius) for r in reference.get(det.frame, [])) for det in track.points)
    return close / len(track.points) >= match.purity


def reference_points(tracks: Sequence[Track]) -> dict[int, list[Detection]]:
    points: dict[int, list[Detection]] = {}
    for track in tracks:
        for det in track.points:
            points.setdefault(det.frame, []).append(det)
    return points
