from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass

from batdetect.detect import Detection


@dataclass(frozen=True, slots=True)
class Span:
    first: int
    last: int

    def contains(self, frame: int) -> bool:
        return self.first <= frame <= self.last


def covered_frames(spans: Iterable[Span]) -> set[int]:
    return {f for s in spans for f in range(s.first, s.last + 1)}


def without_spans(detections: dict[int, list[Detection]], spans: Iterable[Span]) -> dict[int, list[Detection]]:
    covered = covered_frames(spans)
    return {f: [] if f in covered else found for f, found in detections.items()}
