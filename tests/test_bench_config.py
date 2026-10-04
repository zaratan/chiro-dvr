from __future__ import annotations

from collections.abc import Callable

import pytest

from batdetect.bench.config import MatchConfig


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
