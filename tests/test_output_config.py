from __future__ import annotations

from collections.abc import Callable
from typing import cast

import pytest

from batdetect.output.config import Encoder, RenderConfig


@pytest.mark.parametrize(
    "build",
    [
        lambda: RenderConfig(box_pad=-1),
        lambda: RenderConfig(trail_s=-1),
        lambda: RenderConfig(clip_margin_s=-1),
        lambda: RenderConfig(crf=52),
        lambda: RenderConfig(encoder=cast("Encoder", "hevc")),
        lambda: RenderConfig(vt_quality=0),
        lambda: RenderConfig(vt_quality=101),
    ],
)
def test_invalid_render_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match="must be"):
        build()
