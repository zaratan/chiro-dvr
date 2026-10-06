from __future__ import annotations

from batdetect.bench.collect import BenchRun
from batdetect.bench.config import MatchConfig
from batdetect.damage import Probe
from batdetect.detect import Detection
from batdetect.synthetic.injection import Observation
from batdetect.synthetic.trajectory import SyntheticBat
from batdetect.video import VideoInfo
from helpers import detection

INFO = VideoInfo(fps=30.0, frame_count=100, width=480, height=360, work_width=160, work_height=120)


MATCH = MatchConfig(radius=12, purity=0.5)


KEY_INTERVAL = 15


def bat_along(bat_id: int, frames: range, y: float = 60) -> SyntheticBat:
    positions = tuple((90.0 + 18 * k, y * INFO.scale) for k in range(len(frames)))
    return SyntheticBat(bat_id, frames.start, positions, tuple(-40.0 for _ in frames), -40.0, 3.0)


def observations_of(bat: SyntheticBat, effective: float = -40) -> list[Observation]:
    return [Observation(bat.start_frame + k, x, y, effective) for k, (x, y) in enumerate(bat.positions)]


def detections_on(bat: SyntheticBat, skip: set[int] | None = None) -> dict[int, list[Detection]]:
    out: dict[int, list[Detection]] = {}
    for k, (x, y) in enumerate(bat.positions):
        frame = bat.start_frame + k
        if frame not in (skip or set()):
            out[frame] = [detection(frame, x, y)]
    return out


def make_run(
    bats: list[SyntheticBat],
    injected: dict[int, list[Detection]],
    *,
    effective: float = -40,
    damaged: tuple[int, ...] = (),
    reference: dict[int, list[Detection]] | None = None,
) -> BenchRun:
    frames = {f: injected.get(f, []) for f in range(INFO.frame_count)}
    observations = {b.id: observations_of(b, effective) for b in bats}
    probe = Probe(INFO.frame_count, damaged, (), tuple(range(0, INFO.frame_count, KEY_INTERVAL)))
    references = {f: (reference or {}).get(f, []) for f in range(INFO.frame_count)}
    return BenchRun("k", INFO, references, frames, bats, probe, 0, observations)
