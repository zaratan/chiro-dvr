from __future__ import annotations

from dataclasses import dataclass

MAX_CRF = 51
MAX_VT_QUALITY = 100
AUTO = "auto"
VIDEOTOOLBOX = "videotoolbox"
X264 = "x264"
ENCODERS = (AUTO, VIDEOTOOLBOX, X264)


@dataclass(frozen=True, slots=True)
class RenderConfig:
    box_pad: int = 12
    trail_s: float = 1.0
    clip_margin_s: float = 1.5
    crf: int = 20
    encoder: str = AUTO
    vt_quality: int = 65

    def __post_init__(self) -> None:
        if self.box_pad < 0:
            raise ValueError("box_pad must be >= 0")
        if self.trail_s < 0:
            raise ValueError("trail_s must be >= 0")
        if self.clip_margin_s < 0:
            raise ValueError("clip_margin_s must be >= 0")
        if not 0 <= self.crf <= MAX_CRF:
            raise ValueError(f"crf must be between 0 and {MAX_CRF}")
        if self.encoder not in ENCODERS:
            raise ValueError(f"encoder must be one of {', '.join(ENCODERS)}")
        if not 1 <= self.vt_quality <= MAX_VT_QUALITY:
            raise ValueError(f"vt_quality must be between 1 and {MAX_VT_QUALITY}")
