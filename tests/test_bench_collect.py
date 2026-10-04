from __future__ import annotations

from pathlib import Path

from batdetect.bench.collect import collect
from batdetect.bench.config import BenchSetup
from batdetect.detect import DetectConfig
from batdetect.synthetic.sampling import Sampling
from batdetect.synthetic.trajectory import BatClass
from batdetect.track import TrackConfig
from helpers import small_video


def test_collection_runs_both_passes_and_records_each_observation_once(tmp_path: Path) -> None:
    video = small_video(tmp_path / "v.mp4", frames=150)
    setup = BenchSetup(video, DetectConfig(), TrackConfig(), Sampling(5, (BatClass(-80, 4),), 2, 72), workers=1)

    run = collect(setup)

    assert sorted(run.reference) == list(range(150))
    assert sorted(run.injected) == list(range(150))
    assert len(run.bats) == 2
    for bat in run.bats:
        frames = [o.frame for o in run.observations.get(bat.id, [])]
        assert frames == sorted(set(frames))
        assert all(bat.start_frame <= f <= bat.end_frame for f in frames)
    assert run.key == ""
