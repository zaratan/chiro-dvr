from __future__ import annotations

import numpy as np
import pytest

from batdetect.median import IntFrame, temporal_median

INT16 = np.iinfo(np.int16)


def stack(count: int, seed: int, low: int = INT16.min, high: int = INT16.max) -> list[IntFrame]:
    rng = np.random.default_rng(seed)
    return [rng.integers(low, high, (6, 7), endpoint=True).astype(np.int16) for _ in range(count)]


@pytest.mark.parametrize("count", range(1, 32))
def test_matches_numpy_median_bit_for_bit_for_every_stack_size(count: int) -> None:
    frames = stack(count, seed=count)

    result = temporal_median(frames)

    assert result.dtype == np.float64
    assert np.array_equal(result, np.median(np.stack(frames), axis=0))


def test_even_stack_averages_the_two_middle_values_without_int16_overflow() -> None:
    frames = [np.full((2, 2), v, dtype=np.int16) for v in (32000, 32001, -5, 32767)]

    assert temporal_median(frames).tolist() == [[32000.5, 32000.5], [32000.5, 32000.5]]


@pytest.mark.parametrize("count", range(1, 32))
def test_matches_numpy_median_on_eight_bit_frames(count: int) -> None:
    rng = np.random.default_rng(200 + count)
    frames = [rng.integers(0, 255, (6, 7), endpoint=True).astype(np.uint8) for _ in range(count)]

    result = temporal_median(frames)

    assert result.dtype == np.float64
    assert np.array_equal(result, np.median(np.stack(frames), axis=0))


def test_even_eight_bit_stack_averages_without_wrapping_around() -> None:
    frames = [np.full((1, 1), v, dtype=np.uint8) for v in (250, 251)]

    assert temporal_median(frames).tolist() == [[250.5]]


def test_input_frames_are_left_untouched() -> None:
    frames = stack(9, seed=7)
    copies = [f.copy() for f in frames]

    temporal_median(frames)

    assert all(np.array_equal(f, c) for f, c in zip(frames, copies, strict=True))


def test_empty_stack_is_refused() -> None:
    with pytest.raises(ValueError, match="at least one frame"):
        temporal_median([])
