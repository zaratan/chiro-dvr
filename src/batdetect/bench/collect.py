from __future__ import annotations

from dataclasses import dataclass, field

from batdetect.bench.config import BenchSetup
from batdetect.detect import Detection
from batdetect.parallel import detect_video
from batdetect.synthetic.injection import Injector, Observation
from batdetect.synthetic.sampling import random_bats, reference_occupancy
from batdetect.synthetic.trajectory import SyntheticBat
from batdetect.track import track_detections
from batdetect.video import VideoInfo, open_video


@dataclass(slots=True)
class BenchRun:
    key: str
    info: VideoInfo
    reference: dict[int, list[Detection]]
    injected: dict[int, list[Detection]]
    bats: list[SyntheticBat]
    observations: dict[int, list[Observation]] = field(default_factory=dict[int, list[Observation]])


def collect(setup: BenchSetup) -> BenchRun:
    video, detect, sampling = setup.video, setup.detect, setup.sampling
    cap, info = open_video(video, detect.work_width)
    cap.release()
    reference = detect_video(video, detect, info, setup.workers)[0]
    occupied = reference_occupancy(track_detections(reference, setup.track))
    bats = random_bats(sampling, info, len(reference), occupied)
    injected, injectors = detect_video(video, detect, info, setup.workers, lambda: Injector(bats))
    unique: dict[tuple[int, int], Observation] = {}
    for injector in injectors:
        for bat_id, found in injector.observations.items():
            for obs in found:
                unique[bat_id, obs.frame] = obs
    observations: dict[int, list[Observation]] = {}
    for (bat_id, _), obs in sorted(unique.items()):
        observations.setdefault(bat_id, []).append(obs)
    return BenchRun("", info, reference, injected, bats, observations)
