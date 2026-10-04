from __future__ import annotations

from batdetect.bench.metrics import BatResult, Evaluation
from batdetect.bench.stats import summarize, wilson
from batdetect.synthetic.trajectory import Motion


def test_wilson_interval_matches_the_textbook_value() -> None:
    low, high = wilson(5, 10)

    assert (round(low, 3), round(high, 3)) == (0.237, 0.763)


def result(bat_id: int, motion: Motion, found: bool) -> BatResult:
    return BatResult(bat_id, motion, -60.0, 3.0, 20, found, 1.0 if found else 0.0, 0, 0, 1, -40.0)


def test_passes_and_hunts_of_the_same_class_get_their_own_rows_passes_first() -> None:
    evaluation = Evaluation([result(0, "hunt", False), result(1, "pass", True), result(2, "circle", True)], 0, 0, 0, 0)

    rows = summarize(evaluation)

    assert [(r["motion"], r["found"]) for r in rows] == [("pass", 1), ("hunt", 0), ("circle", 1)]
