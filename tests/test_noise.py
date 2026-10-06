from __future__ import annotations

import numpy as np
import numpy.typing as npt
import pytest

from batdetect.median import temporal_median
from batdetect.noise import MAD_TO_SIGMA, temporal_noise
from batdetect.video import GrayFrame


def noisy_samples(count: int, sigma: float, seed: int) -> list[GrayFrame]:
    rng = np.random.default_rng(seed)
    return [np.clip(rng.normal(120, sigma, (40, 50)), 0, 255).astype(np.uint8) for _ in range(count)]


def gains_of(samples: list[GrayFrame]) -> list[float]:
    levels = [float(s.mean()) for s in samples]
    return [level - float(np.mean(levels)) for level in levels]


def numpy_noise(
    samples: list[GrayFrame], background: npt.NDArray[np.float64], gains: list[float]
) -> npt.NDArray[np.float64]:
    deviations = np.stack([s - background - g for s, g in zip(samples, gains, strict=True)])
    return MAD_TO_SIGMA * np.median(np.abs(deviations - np.median(deviations, axis=0)), axis=0)


@pytest.mark.parametrize("count", [7, 8, 11])
def test_noise_map_is_the_scaled_median_absolute_deviation(count: int) -> None:
    samples = noisy_samples(count, sigma=6, seed=count)
    background = temporal_median(samples)
    gains = gains_of(samples)

    noise = temporal_noise(samples, background, gains)

    assert np.allclose(noise, numpy_noise(samples, background, gains), rtol=0, atol=1e-4)


def test_noise_map_follows_the_pixel_noise_level() -> None:
    samples = noisy_samples(11, sigma=6, seed=1)

    noise = temporal_noise(samples, temporal_median(samples), gains_of(samples))

    assert float(np.median(noise)) == pytest.approx(6, rel=0.25)


def test_global_gain_jump_does_not_inflate_the_noise_map() -> None:
    calm = noisy_samples(11, sigma=3, seed=2)
    jumped = [s if k < 5 else (s + 30).astype(np.uint8) for k, s in enumerate(calm)]

    steady = temporal_noise(calm, temporal_median(calm), gains_of(calm))
    after_jump = temporal_noise(jumped, temporal_median(jumped), gains_of(jumped))

    assert float(np.median(after_jump)) == pytest.approx(float(np.median(steady)), abs=1)
