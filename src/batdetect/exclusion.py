from __future__ import annotations

from dataclasses import dataclass

from batdetect.damage import Probe, check_frame_count, damaged_spans
from batdetect.detect import Detection
from batdetect.spans import Span, covered_frames, without_spans
from batdetect.stability import StabilityConfig, unstable_spans


@dataclass(frozen=True, slots=True)
class Analysis:
    detections: dict[int, list[Detection]]
    damaged: list[Span]
    unstable: list[Span]
    reported_frames: int

    @property
    def ignored_spans(self) -> list[Span]:
        return [*self.damaged, *self.unstable]

    @property
    def ignored_frames(self) -> set[int]:
        return covered_frames(self.ignored_spans)


def exclude(
    detections: dict[int, list[Detection]], probe: Probe, margin: int, fps: float, stability: StabilityConfig
) -> Analysis:
    check_frame_count(probe.frame_count, len(detections))
    damaged = damaged_spans(probe, margin)
    hidden = covered_frames(damaged)
    readable = {f: found for f, found in detections.items() if f not in hidden}
    unstable = unstable_spans(readable, fps, stability)
    return Analysis(without_spans(detections, [*damaged, *unstable]), damaged, unstable, len(probe.damaged_starts))
