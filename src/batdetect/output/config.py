from __future__ import annotations

from dataclasses import dataclass

MAX_CRF = 51


@dataclass(frozen=True, slots=True)
class RenderConfig:
    box_pad: int = 12
    trail_s: float = 1.0
    clip_margin_s: float = 1.5
    crf: int = 20

    def __post_init__(self) -> None:
        if self.box_pad < 0:
            raise ValueError("box_pad must be >= 0")
        if self.trail_s < 0:
            raise ValueError("trail_s must be >= 0")
        if self.clip_margin_s < 0:
            raise ValueError("clip_margin_s must be >= 0")
        if not 0 <= self.crf <= MAX_CRF:
            raise ValueError(f"crf must be between 0 and {MAX_CRF}")
