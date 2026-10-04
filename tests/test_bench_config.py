from __future__ import annotations

from collections.abc import Callable

import pytest

from batdetect.bench.config import MatchConfig, bench_classes


@pytest.mark.parametrize(
    "build",
    [
        lambda: MatchConfig(radius=0),
        lambda: MatchConfig(purity=0),
        lambda: MatchConfig(purity=1.5),
    ],
)
def test_invalid_match_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match="must be"):
        build()


def test_full_purity_is_allowed() -> None:
    assert MatchConfig(purity=1).purity == 1


def test_classes_are_drawn_passes_first_whatever_the_order_typed_and_without_duplicates() -> None:
    typed = bench_classes(["hunt", "pass", "pass"], [-60.0], [3.0])

    assert typed == bench_classes(["pass", "hunt"], [-60.0], [3.0])
    assert [c.motion for c in typed] == ["pass", "hunt"]
