from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from batdetect.detect import DetectConfig
from batdetect.stability import StabilityConfig
from batdetect.synthetic.sampling import Sampling
from batdetect.track import TrackConfig

DEFAULT_AMPLITUDES = (-15.0, -30.0, -60.0)


DEFAULT_SIGMAS = (1.5, 3.0, 5.0)


@dataclass(frozen=True, slots=True)
class MatchConfig:
    radius: float = 24.0
    purity: float = 0.5

    def __post_init__(self) -> None:
        if self.radius <= 0:
            raise ValueError("radius must be > 0")
        if not 0 < self.purity <= 1:
            raise ValueError("purity must be in (0, 1]")


@dataclass(frozen=True, slots=True)
class BenchSetup:
    video: Path
    detect: DetectConfig
    track: TrackConfig
    stability: StabilityConfig
    sampling: Sampling
    workers: int
