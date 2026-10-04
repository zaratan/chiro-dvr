from __future__ import annotations

import numpy as np

from batdetect.synthetic.hunting import MAX_DURATION_S, make_hunting_bat
from batdetect.synthetic.sampling import Sampling, draw_shape, random_bats
from batdetect.synthetic.trajectory import BatClass
from batdetect.video import VideoInfo

INFO = VideoInfo(fps=30.0, frame_count=600, width=480, height=360, work_width=160, work_height=120)


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


def test_adding_hunting_classes_after_the_passes_leaves_every_pass_unchanged() -> None:
    passes = Sampling(7, (BatClass(-30, 2), BatClass(-60, 3)), 4, min_distance=8)
    both = Sampling(7, (BatClass(-30, 2), BatClass(-60, 3), BatClass(-60, 3, "hunt")), 4, min_distance=8)

    before = random_bats(passes, INFO, 600, {})
    after = random_bats(both, INFO, 600, {})

    assert [(b.start_frame, b.positions, b.amplitudes) for b in before] == [
        (b.start_frame, b.positions, b.amplitudes) for b in after[:8]
    ]
    assert {b.motion for b in after[8:]} == {"hunt"}
    assert all(len(b.positions) <= round(MAX_DURATION_S * INFO.fps) for b in after[8:])


def test_hunting_and_circling_classes_draw_hunting_flights_not_crossings() -> None:
    for motion in ("hunt", "circle"):
        kind = BatClass(-60, 3, motion)

        drawn = draw_shape(0, kind, INFO, np.random.default_rng(5))

        assert drawn.positions == make_hunting_bat(0, 0, kind, INFO, np.random.default_rng(5)).positions
