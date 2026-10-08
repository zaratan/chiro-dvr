from __future__ import annotations

import itertools
import math
import statistics
from dataclasses import dataclass, field

from batdetect.detect import Detection

VELOCITY_SPAN_FRAMES = 3


@dataclass(frozen=True, slots=True)
class TrackConfig:
    max_jump: float = 120
    max_gap: int = 6
    min_hits: int = 6
    min_travel: float = 45
    twin_distance: float = 36
    max_median_turn: float = 0.8

    def __post_init__(self) -> None:
        if self.max_jump <= 0:
            raise ValueError("max_jump must be > 0")
        if self.max_gap < 1:
            raise ValueError("max_gap must be >= 1")
        if self.min_hits < 1:
            raise ValueError("min_hits must be >= 1")
        if self.min_travel < 0:
            raise ValueError("min_travel must be >= 0")
        if self.twin_distance < 0:
            raise ValueError("twin_distance must be >= 0")
        if not self.max_median_turn > 0:
            raise ValueError("max_median_turn must be > 0")


@dataclass(slots=True)
class Track:
    id: int
    points: list[Detection] = field(default_factory=list[Detection])

    @property
    def first(self) -> Detection:
        return self.points[0]

    @property
    def last(self) -> Detection:
        return self.points[-1]

    def predicted(self, frame: int) -> tuple[float, float]:
        last = self.last
        if len(self.points) == 1:
            return last.x, last.y
        prev = next(
            (p for p in reversed(self.points[:-1]) if last.frame - p.frame >= VELOCITY_SPAN_FRAMES), self.points[-2]
        )
        dt = last.frame - prev.frame
        ahead = frame - last.frame
        return last.x + (last.x - prev.x) / dt * ahead, last.y + (last.y - prev.y) / dt * ahead

    def chord(self) -> float:
        return math.hypot(self.last.x - self.first.x, self.last.y - self.first.y)

    def median_turn(self) -> float:
        headings = [
            math.atan2(b.y - a.y, b.x - a.x) for a, b in itertools.pairwise(self.points) if (a.x, a.y) != (b.x, b.y)
        ]
        turns = [abs((b - a + math.pi) % (2 * math.pi) - math.pi) for a, b in itertools.pairwise(headings)]
        return statistics.median(turns) if turns else 0.0

    def path_length(self) -> float:
        return sum(math.hypot(b.x - a.x, b.y - a.y) for a, b in zip(self.points, self.points[1:], strict=False))

    def max_area(self) -> float:
        return max(p.area for p in self.points)

    def peak_amplitude(self) -> float:
        return max(p.amplitude for p in self.points)


def _match(active: list[Track], candidates: list[Detection], frame: int, max_jump: float) -> list[Detection]:
    pairs: list[tuple[bool, float, int, int]] = []
    for ti, track in enumerate(active):
        px, py = track.predicted(frame)
        has_velocity = len(track.points) > 1
        for di, det in enumerate(candidates):
            dist = math.hypot(det.x - px, det.y - py)
            if dist <= max_jump:
                pairs.append((not has_velocity, dist, ti, di))
    used_tracks: set[int] = set()
    used_dets: set[int] = set()
    for _, _, ti, di in sorted(pairs):
        if ti in used_tracks or di in used_dets:
            continue
        active[ti].points.append(candidates[di])
        used_tracks.add(ti)
        used_dets.add(di)
    return [d for di, d in enumerate(candidates) if di not in used_dets]


def track_detections(detections: dict[int, list[Detection]], cfg: TrackConfig) -> list[Track]:
    active: list[Track] = []
    finished: list[Track] = []
    for frame in sorted(detections):
        finished.extend(t for t in active if frame - t.last.frame - 1 > cfg.max_gap)
        active = [t for t in active if frame - t.last.frame - 1 <= cfg.max_gap]
        unmatched = _match(active, detections[frame], frame, cfg.max_jump)
        active.extend(Track(0, [d]) for d in unmatched)
    finished.extend(active)
    kept = [
        t
        for t in merge_twins(finished, cfg.twin_distance)
        if len(t.points) >= cfg.min_hits and t.chord() >= cfg.min_travel and t.median_turn() <= cfg.max_median_turn
    ]
    kept.sort(key=lambda t: (t.first.frame, t.first.x))
    for number, track in enumerate(kept, 1):
        track.id = number
    return kept


def are_twins(a: Track, b: Track, max_distance: float) -> bool:
    if a.last.frame < b.first.frame or b.last.frame < a.first.frame:
        return False
    by_frame = {p.frame: p for p in a.points}
    common = [(by_frame[p.frame], p) for p in b.points if p.frame in by_frame]
    return bool(common) and all(math.hypot(pa.x - pb.x, pa.y - pb.y) <= max_distance for pa, pb in common)


def combine(a: Detection, b: Detection) -> Detection:
    left, top = min(a.left, b.left), min(a.top, b.top)
    right = max(a.left + a.width, b.left + b.width)
    bottom = max(a.top + a.height, b.top + b.height)
    area = a.area + b.area
    x = (a.x * a.area + b.x * b.area) / area
    y = (a.y * a.area + b.y * b.area) / area
    return Detection(a.frame, x, y, left, top, right - left, bottom - top, area, max(a.amplitude, b.amplitude))


def merge_twins(tracks: list[Track], max_distance: float) -> list[Track]:
    if max_distance <= 0:
        return tracks
    ordered = sorted(tracks, key=lambda t: t.first.frame)
    parent = list(range(len(ordered)))

    def root(i: int) -> int:
        while parent[i] != i:
            parent[i] = parent[parent[i]]
            i = parent[i]
        return i

    for i, a in enumerate(ordered):
        for j in range(i + 1, len(ordered)):
            b = ordered[j]
            if b.first.frame > a.last.frame:
                break
            if are_twins(a, b, max_distance):
                parent[root(j)] = root(i)

    groups: dict[int, list[Track]] = {}
    for i, track in enumerate(ordered):
        groups.setdefault(root(i), []).append(track)
    merged: list[Track] = []
    for members in groups.values():
        by_frame: dict[int, Detection] = {}
        for det in (p for t in members for p in t.points):
            by_frame[det.frame] = combine(by_frame[det.frame], det) if det.frame in by_frame else det
        merged.append(Track(0, [by_frame[f] for f in sorted(by_frame)]))
    return merged
