from __future__ import annotations

import bisect
import itertools
import json
import statistics
from dataclasses import dataclass
from typing import cast

from batdetect.spans import Span
from batdetect.video import VideoError

PTS_GAP_FACTOR = 1.5


@dataclass(frozen=True, slots=True)
class Probe:
    frame_count: int
    error_frames: tuple[int, ...]
    gap_frames: tuple[int, ...]
    key_frames: tuple[int, ...]

    @property
    def damaged_starts(self) -> list[int]:
        return sorted({*self.error_frames, *self.gap_frames})


def frame_entries(text: str) -> list[dict[str, object]]:
    try:
        document: object = json.loads(text)
    except json.JSONDecodeError as err:
        raise VideoError(f"ffprobe output is not JSON: {err}") from err
    if not isinstance(document, dict):
        raise VideoError("ffprobe output is not a JSON object")
    frames: object = cast(dict[str, object], document).get("frames", [])
    if not isinstance(frames, list) or not all(isinstance(entry, dict) for entry in cast(list[object], frames)):
        raise VideoError("ffprobe output has no valid frame list")
    return cast(list[dict[str, object]], frames)


def frames_after_pts_gaps(pts: list[int | None]) -> list[int]:
    known = [(k, p) for k, p in enumerate(pts) if p is not None]
    pairs = list(itertools.pairwise(known))
    steps = [(b - a) / (kb - ka) for (ka, a), (kb, b) in pairs]
    if not steps or (step := statistics.median(steps)) <= 0:
        return []
    return [kb for (ka, a), (kb, b) in pairs if b - a > PTS_GAP_FACTOR * step * (kb - ka)]


def parse_probe(text: str) -> Probe:
    frames = frame_entries(text)
    pts = [p if isinstance(p := f.get("pts"), int) else None for f in frames]
    return Probe(
        frame_count=len(frames),
        error_frames=tuple(k for k, f in enumerate(frames) if f.get("logs")),
        gap_frames=tuple(frames_after_pts_gaps(pts)),
        key_frames=tuple(k for k, f in enumerate(frames) if f.get("key_frame") == 1),
    )


def damaged_spans(probe: Probe, margin: int) -> list[Span]:
    spans: list[Span] = []
    for start in probe.damaged_starts:
        next_key = bisect.bisect_right(probe.key_frames, start)
        stop = probe.key_frames[next_key] - 1 if next_key < len(probe.key_frames) else probe.frame_count - 1
        first, last = max(0, start - margin), min(probe.frame_count - 1, stop + margin)
        if spans and first <= spans[-1].last + 1:
            spans[-1] = Span(spans[-1].first, max(spans[-1].last, last))
        else:
            spans.append(Span(first, last))
    return spans


def check_frame_count(probed: int, read: int) -> None:
    if probed != read:
        raise VideoError(f"ffprobe decoded {probed} frames but OpenCV read {read}: frame numbers do not match")
