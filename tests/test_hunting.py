from __future__ import annotations

import math

import numpy as np
import pytest

from batdetect.synthetic.hunting import (
    CRUISE_SPEED_RANGE,
    MAX_DURATION_S,
    STRAIGHT_DRIFT,
    hunting_path,
    make_hunting_bat,
    turn_profile,
    with_saccade,
)
from batdetect.synthetic.trajectory import SACCADE_PERIOD, BatClass, Motion, Point
from batdetect.video import VideoInfo

INFO = VideoInfo(fps=30.0, frame_count=9000, width=1440, height=1080, work_width=480, work_height=360)
SEEDS = range(40)


def turns(points: list[Point] | tuple[Point, ...]) -> np.ndarray[tuple[int], np.dtype[np.float64]]:
    steps = np.diff(np.array(points), axis=0)
    headings = np.arctan2(steps[:, 1], steps[:, 0])
    return np.abs((np.diff(headings) + np.pi) % (2 * np.pi) - np.pi)


def sharpest_turn(points: list[Point] | tuple[Point, ...], window: int = 20) -> float:
    steps = np.diff(np.array(points), axis=0)
    headings = np.unwrap(np.arctan2(steps[:, 1], steps[:, 0]))
    span = min(window, len(headings) - 1)
    return float(max(abs(headings[k + span] - headings[k]) for k in range(len(headings) - span)))


def test_a_turn_turns_by_the_requested_angle_and_is_tightest_at_its_apex() -> None:
    turn = turn_profile(cruise=20.0, apex_ratio=0.25, apex_rate=0.3, angle=math.pi)
    speeds, rates = zip(*turn, strict=True)

    assert sum(rates) == pytest.approx(math.pi)
    assert min(speeds) < 0.3 * 20.0
    assert rates.index(max(rates)) == speeds.index(min(speeds))


def test_every_hunting_path_enters_from_outside_heading_inward_and_really_turns() -> None:
    center = np.array([INFO.width / 2, INFO.height / 2])
    for seed in SEEDS:
        path = hunting_path(np.random.default_rng(seed), BatClass(-60, 3, "hunt"), INFO)
        x, y = path[0]

        assert not (0 < x < INFO.width and 0 < y < INFO.height)
        assert np.linalg.norm(np.array(path[1]) - center) < np.linalg.norm(np.array(path[0]) - center)
        assert sharpest_turn(path) >= math.pi / 4


def test_hunting_paths_slow_down_in_turns_and_stop_within_the_time_limit() -> None:
    for seed in SEEDS:
        path = hunting_path(np.random.default_rng(seed), BatClass(-60, 3, "hunt"), INFO)
        steps = np.hypot(*np.diff(np.array(path), axis=0).T)

        assert steps.max() <= CRUISE_SPEED_RANGE[1]
        assert steps.min() < 0.6 * steps.max()
        assert len(path) <= round(MAX_DURATION_S * INFO.fps)


def test_circling_spends_more_time_turning_than_hunting() -> None:
    def turning_share(motion: Motion) -> float:
        shares = [
            float(
                (
                    turns(hunting_path(np.random.default_rng(seed), BatClass(-60, 3, motion), INFO)) > STRAIGHT_DRIFT
                ).mean()
            )
            for seed in SEEDS
        ]
        return float(np.median(shares))

    assert turning_share("circle") > 0.5
    assert turning_share("circle") > turning_share("hunt")


def test_saccade_keeps_the_ends_and_doubles_one_step_in_three() -> None:
    path = [(float(k), 0.0) for k in range(13)]

    resampled = with_saccade(path, phase=0)
    steps = np.diff([x for x, _ in resampled])

    assert len(resampled) == 13
    assert (resampled[0], resampled[-1]) == (path[0], path[-1])
    assert steps[2] == pytest.approx(2 * steps[0])
    assert steps[5] == pytest.approx(2 * steps[3])


def test_a_hunting_bat_is_reproducible_and_carries_its_motion() -> None:
    kind = BatClass(-60, 3, "circle")

    one = make_hunting_bat(4, 100, kind, INFO, np.random.default_rng(9))
    two = make_hunting_bat(4, 100, kind, INFO, np.random.default_rng(9))

    assert one == two
    assert one.motion == "circle"
    assert len(one.amplitudes) == len(one.positions)


def test_a_hunting_bat_follows_its_path_with_the_saccade() -> None:
    kind = BatClass(-60, 3, "hunt")
    rng = np.random.default_rng(9)
    path = hunting_path(rng, kind, INFO)
    expected = with_saccade(path, int(rng.integers(SACCADE_PERIOD)))

    assert make_hunting_bat(0, 0, kind, INFO, np.random.default_rng(9)).positions == expected
