from __future__ import annotations

import math
from collections.abc import Iterable
from dataclasses import dataclass, field

import numpy as np

from batdetect.synthetic.trajectory import SyntheticBat
from batdetect.video import ColorFrame


def inject(frame: ColorFrame, bats: Iterable[SyntheticBat], frame_no: int) -> dict[int, float]:
    height, width = frame.shape[:2]
    effective: dict[int, float] = {}
    for bat in bats:
        pos = bat.position(frame_no)
        if pos is None:
            continue
        x, y = pos
        radius = math.ceil(3 * bat.sigma)
        x0, x1 = max(0, math.floor(x) - radius), min(width, math.floor(x) + radius + 1)
        y0, y1 = max(0, math.floor(y) - radius), min(height, math.floor(y) + radius + 1)
        if x0 >= x1 or y0 >= y1:
            continue
        ys, xs = np.mgrid[y0:y1, x0:x1]
        blob = bat.amplitude_at(frame_no) * np.exp(-((xs - x) ** 2 + (ys - y) ** 2) / (2 * bat.sigma**2))
        before = frame[y0:y1, x0:x1].astype(np.float32)
        after = np.clip(before + blob[..., None], 0, 255)
        frame[y0:y1, x0:x1] = after.astype(np.uint8)
        change = (after - before)[..., 0]
        effective[bat.id] = float(change.min() if bat.amplitude < 0 else change.max())
    return effective


@dataclass(frozen=True, slots=True)
class Observation:
    frame: int
    x: float
    y: float
    effective: float


@dataclass(slots=True)
class Injector:
    bats: list[SyntheticBat]
    observations: dict[int, list[Observation]] = field(default_factory=dict[int, list[Observation]])
    active: dict[int, list[SyntheticBat]] = field(default_factory=dict[int, list[SyntheticBat]])

    def __post_init__(self) -> None:
        for bat in self.bats:
            for frame in range(bat.start_frame, bat.end_frame + 1):
                self.active.setdefault(frame, []).append(bat)

    def __call__(self, frame: ColorFrame, frame_no: int) -> None:
        here = self.active.get(frame_no, [])
        effective = inject(frame, here, frame_no)
        for bat in here:
            pos = bat.position(frame_no)
            if pos is not None and bat.id in effective:
                self.observations.setdefault(bat.id, []).append(Observation(frame_no, *pos, effective[bat.id]))
