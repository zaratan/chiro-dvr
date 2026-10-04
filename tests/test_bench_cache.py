from __future__ import annotations

from pathlib import Path

from batdetect.bench.cache import cache_key
from batdetect.detect import DetectConfig
from batdetect.synthetic.sampling import Sampling
from batdetect.synthetic.trajectory import BatClass


def test_cache_key_ignores_int_versus_float_spelling(tmp_path: Path) -> None:
    video = tmp_path / "v.mp4"
    video.write_bytes(b"x")
    sampling = Sampling(1, (BatClass(-30, 2),), 3, 72)

    assert cache_key(video, DetectConfig(min_area=4), sampling) == cache_key(
        video, DetectConfig(min_area=4.0), sampling
    )
