from __future__ import annotations

import itertools
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from batdetect.output.timefmt import clip_name
from batdetect.track import Track
from batdetect.video import ColorFrame

type View = Callable[[ColorFrame, int], ColorFrame]


@dataclass(frozen=True, slots=True)
class ClipWindow:
    first: int
    last: int
    path: Path
    view: View | None = None
    slowdown: int = 1


def clip_windows(tracks: list[Track], fps: float, margin_s: float, split_dir: Path) -> list[ClipWindow]:
    margin = round(margin_s * fps)
    return [
        ClipWindow(max(0, t.first.frame - margin), t.last.frame + margin, split_dir / clip_name(t, fps)) for t in tracks
    ]


def in_passes(windows: list[ClipWindow], max_writers: int) -> list[list[ClipWindow]]:
    if max_writers < 1:
        raise ValueError("max_writers must be >= 1")
    lanes: list[list[ClipWindow]] = []
    for window in sorted(windows, key=lambda w: w.first):
        free = next((lane for lane in lanes if lane[-1].last < window.first), None)
        if free is None:
            lanes.append([window])
        else:
            free.append(window)
    return [
        sorted((w for lane in group for w in lane), key=lambda w: w.first)
        for group in itertools.batched(lanes, max_writers, strict=False)
    ]
