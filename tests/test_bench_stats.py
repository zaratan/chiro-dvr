from __future__ import annotations

from batdetect.bench.stats import wilson


def test_wilson_interval_matches_the_textbook_value() -> None:
    low, high = wilson(5, 10)

    assert (round(low, 3), round(high, 3)) == (0.237, 0.763)
