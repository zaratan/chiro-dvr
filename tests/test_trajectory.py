from __future__ import annotations

import itertools

import numpy as np
import pytest

from batdetect.synthetic.trajectory import BatClass, Flight, make_bat, saccade_progress


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
