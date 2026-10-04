from __future__ import annotations

import math
import statistics
from dataclasses import dataclass

from batdetect.detect import Detection

TYPICAL_FACTOR = 6


@dataclass(frozen=True, slots=True)
class StabilityConfig:
    max_blobs: int = 20
    pad_s: float = 1.0

    def __post_init__(self) -> None:
        if self.max_blobs < 0:
            raise ValueError("max_blobs must be >= 0")
        if not (math.isfinite(self.pad_s) and self.pad_s >= 0):
            raise ValueError("pad_s must be a finite number >= 0")


@dataclass(frozen=True, slots=True)
class Span:
    first: int
    last: int

    def contains(self, frame: int) -> bool:
        return self.first <= frame <= self.last


def crowd_limit(counts: list[int], max_blobs: int) -> float:
    return max_blobs + TYPICAL_FACTOR * statistics.median(counts)


def unstable_spans(detections: dict[int, list[Detection]], fps: float, cfg: StabilityConfig) -> list[Span]:
    if cfg.max_blobs == 0 or not detections:
        return []
    frames = sorted(detections)
    limit = crowd_limit([len(detections[f]) for f in frames], cfg.max_blobs)
    pad = round(cfg.pad_s * fps)
    spans: list[Span] = []
    for frame in (f for f in frames if len(detections[f]) >= limit):
        first, last = max(frames[0], frame - pad), min(frames[-1], frame + pad)
        if spans and first <= spans[-1].last + pad + 1:
            spans[-1] = Span(spans[-1].first, last)
        else:
            spans.append(Span(first, last))
    return spans


def without_spans(detections: dict[int, list[Detection]], spans: list[Span]) -> dict[int, list[Detection]]:
    return {f: [] if any(s.contains(f) for s in spans) else found for f, found in detections.items()}
