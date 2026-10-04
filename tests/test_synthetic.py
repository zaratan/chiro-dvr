from __future__ import annotations

import itertools

import numpy as np
import pytest

from batdetect.pipeline import VideoInfo
from batdetect.synthetic import (
    BatClass,
    Flight,
    Sampling,
    SyntheticBat,
    inject,
    make_bat,
    random_bats,
    saccade_progress,
)

INFO = VideoInfo(fps=30.0, frame_count=600, width=480, height=360, work_width=160, work_height=120)


def still_bat(amplitude: float, sigma: float, x: float = 100, y: float = 100) -> SyntheticBat:
    return SyntheticBat(0, 0, ((x, y),), (amplitude,), amplitude, sigma)


def test_saccade_doubles_one_step_in_three() -> None:
    progress = saccade_progress(10, phase=0)
    steps = [b - a for a, b in itertools.pairwise(progress)]

    assert progress[0] == 0
    assert progress[-1] == pytest.approx(1)
    assert steps[2] == pytest.approx(2 * steps[0])
    assert steps[5] == pytest.approx(2 * steps[3])


def test_bat_flies_from_entry_to_exit_and_accelerates() -> None:
    flight = Flight(((0, 0), (50, 0), (100, 0)), speed=5, acceleration=1.8)

    bat = make_bat(3, 10, flight, BatClass(-40, 2), np.random.default_rng(0))

    assert bat.positions[0] == pytest.approx((0, 0))
    assert bat.positions[-1] == pytest.approx((100, 0))
    first_step = bat.positions[3][0] - bat.positions[0][0]
    last_step = bat.positions[-1][0] - bat.positions[-4][0]
    assert last_step > 2 * first_step
    assert bat.position(9) is None
    assert bat.position(10) == bat.positions[0]


def test_injected_dark_blob_lowers_the_centre_by_its_amplitude() -> None:
    frame = np.full((200, 200, 3), 200, dtype=np.uint8)

    effective = inject(frame, [still_bat(-60, 2)], 0)

    assert effective[0] == pytest.approx(-60, abs=1)
    assert int(frame[100, 100, 0]) == 140
    assert int(frame[10, 10, 0]) == 200


def test_dark_blob_on_saturated_sky_has_no_effective_contrast() -> None:
    frame = np.zeros((200, 200, 3), dtype=np.uint8)

    effective = inject(frame, [still_bat(-60, 2)], 0)

    assert effective[0] == 0


def test_bat_outside_the_frame_is_not_injected() -> None:
    frame = np.full((50, 50, 3), 200, dtype=np.uint8)

    assert inject(frame, [still_bat(-60, 2, x=-40, y=-40)], 0) == {}


def test_sampling_is_reproducible_and_stable_when_a_class_is_added() -> None:
    one = Sampling(7, (BatClass(-30, 2),), 4, min_distance=8)
    two = Sampling(7, (BatClass(-30, 2), BatClass(-60, 3)), 4, min_distance=8)

    first = random_bats(one, INFO, 600, {})
    again = random_bats(one, INFO, 600, {})
    extended = random_bats(two, INFO, 600, {})

    assert [b.positions for b in first] == [b.positions for b in again]
    assert [b.start_frame for b in first] == [b.start_frame for b in extended[:4]]
    assert len(extended) == 8


def test_sampled_bats_keep_away_from_occupied_positions() -> None:
    occupied = {f: [(240.0, 180.0)] for f in range(600)}
    sampling = Sampling(3, (BatClass(-30, 2),), 10, min_distance=20)

    bats = random_bats(sampling, INFO, 600, occupied)

    for bat in bats:
        for x, y in bat.positions:
            assert np.hypot(x - 240, y - 180) >= 20
