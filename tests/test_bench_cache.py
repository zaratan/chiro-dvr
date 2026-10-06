from __future__ import annotations

from pathlib import Path

import pytest

from batdetect.bench.cache import cache_key
from batdetect.detect import DetectConfig
from batdetect.stability import StabilityConfig
from batdetect.synthetic.sampling import Sampling
from batdetect.synthetic.trajectory import BatClass
from helpers import requires_ffmpeg


@requires_ffmpeg
def test_cache_key_ignores_int_versus_float_spelling(tmp_path: Path) -> None:
    video = tmp_path / "v.mp4"
    video.write_bytes(b"x")
    sampling = Sampling(1, (BatClass(-30, 2),), 3, 72)

    assert cache_key(video, DetectConfig(min_area=4), sampling, StabilityConfig()) == cache_key(
        video, DetectConfig(min_area=4.0), sampling, StabilityConfig()
    )


@requires_ffmpeg
def test_cache_key_changes_with_the_stability_settings_since_they_move_the_targets(tmp_path: Path) -> None:
    video = tmp_path / "v.mp4"
    video.write_bytes(b"x")
    sampling = Sampling(1, (BatClass(-30, 2),), 3, 72)

    assert cache_key(video, DetectConfig(), sampling, StabilityConfig()) != cache_key(
        video, DetectConfig(), sampling, StabilityConfig(max_blobs=0)
    )


@requires_ffmpeg
def test_cache_key_changes_with_the_ffprobe_version_since_it_finds_the_damage(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    video = tmp_path / "v.mp4"
    video.write_bytes(b"x")
    sampling = Sampling(1, (BatClass(-30, 2),), 3, 72)
    before = cache_key(video, DetectConfig(), sampling, StabilityConfig())

    monkeypatch.setattr("batdetect.bench.cache.ffprobe_version", lambda: "ffprobe version 6.1")

    assert cache_key(video, DetectConfig(), sampling, StabilityConfig()) != before
