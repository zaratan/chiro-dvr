from __future__ import annotations

import math

import numpy as np
import pytest

from batdetect.output.arrows import arrow_head


def test_arrow_tip_is_the_last_point_and_wings_point_backwards() -> None:
    head = arrow_head(np.array([(0, 0), (100, 0)], dtype=np.float64), 20)

    assert head is not None
    assert head[0].tolist() == [100, 0]
    assert all(x < 100 for x in head[1:, 0])
    assert sorted(np.sign(head[1:, 1]).tolist()) == [-1, 1]


def test_arrow_keeps_its_size_when_the_last_step_is_tiny() -> None:
    head = arrow_head(np.array([(0, 0), (100, 0), (100.5, 0)], dtype=np.float64), 20)

    assert head is not None
    assert math.dist(head[0], head[1]) == pytest.approx(20)


def test_track_that_never_moves_has_no_arrow() -> None:
    assert arrow_head(np.array([(5, 5), (5, 5)], dtype=np.float64), 20) is None
