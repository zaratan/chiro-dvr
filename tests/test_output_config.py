from __future__ import annotations

from collections.abc import Callable

import pytest

from batdetect.output.config import RenderConfig


@pytest.mark.parametrize(
    "build",
    [
        lambda: RenderConfig(box_pad=-1),
        lambda: RenderConfig(trail_s=-1),
        lambda: RenderConfig(clip_margin_s=-1),
        lambda: RenderConfig(crf=52),
    ],
)
def test_invalid_render_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match="must be"):
        build()
