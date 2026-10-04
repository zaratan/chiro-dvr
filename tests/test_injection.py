from __future__ import annotations

import numpy as np
import pytest

from batdetect.synthetic.injection import inject
from batdetect.synthetic.trajectory import SyntheticBat


def still_bat(amplitude: float, sigma: float, x: float = 100, y: float = 100) -> SyntheticBat:
    return SyntheticBat(0, 0, ((x, y),), (amplitude,), amplitude, sigma)


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
