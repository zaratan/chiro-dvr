from __future__ import annotations

import math
from collections.abc import Sequence
from dataclasses import dataclass

from batdetect.track import Track
from batdetect.video import VideoInfo

SUMMARY_SPAN_S = 600.0

type TimeRange = tuple[float, float]


@dataclass(frozen=True, slots=True)
class Period:
    start_s: float
    end_s: float
    tracks: list[Track]
    alone: bool
    ignored: tuple[TimeRange, ...] = ()

    @property
    def suffix(self) -> str:
        if self.alone:
            return ""
        return f"_{math.floor(self.start_s / 60):03d}m-{math.ceil(self.end_s / 60):03d}m"


def clipped(ranges: Sequence[TimeRange], start: float, end: float) -> tuple[TimeRange, ...]:
    return tuple((max(a, start), min(b, end)) for a, b in ranges if a < end and b > start)


def split_by_period(
    tracks: list[Track], info: VideoInfo, span_s: float = SUMMARY_SPAN_S, ignored: Sequence[TimeRange] = ()
) -> list[Period]:
    duration = info.frame_count / info.fps
    if duration <= span_s:
        return [Period(0.0, duration, tracks, alone=True, ignored=clipped(ignored, 0.0, duration))]
    count = math.ceil(duration / span_s)
    groups: list[list[Track]] = [[] for _ in range(count)]
    for track in tracks:
        groups[min(count - 1, math.floor(track.first.frame / info.fps / span_s))].append(track)
    periods: list[Period] = []
    for k, group in enumerate(groups):
        start, end = k * span_s, min((k + 1) * span_s, duration)
        periods.append(Period(start, end, group, alone=False, ignored=clipped(ignored, start, end)))
    return periods
