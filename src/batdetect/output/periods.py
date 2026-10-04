from __future__ import annotations

import math
from dataclasses import dataclass

from batdetect.track import Track
from batdetect.video import VideoInfo

SUMMARY_SPAN_S = 600.0


@dataclass(frozen=True, slots=True)
class Period:
    start_s: float
    end_s: float
    tracks: list[Track]
    alone: bool

    @property
    def suffix(self) -> str:
        if self.alone:
            return ""
        return f"_{math.floor(self.start_s / 60):03d}m-{math.ceil(self.end_s / 60):03d}m"


def split_by_period(tracks: list[Track], info: VideoInfo, span_s: float = SUMMARY_SPAN_S) -> list[Period]:
    duration = info.frame_count / info.fps
    if duration <= span_s:
        return [Period(0.0, duration, tracks, alone=True)]
    count = math.ceil(duration / span_s)
    groups: list[list[Track]] = [[] for _ in range(count)]
    for track in tracks:
        groups[min(count - 1, math.floor(track.first.frame / info.fps / span_s))].append(track)
    return [Period(k * span_s, min((k + 1) * span_s, duration), g, alone=False) for k, g in enumerate(groups)]
