from __future__ import annotations

from collections.abc import Callable
from typing import cast

import pytest

from batdetect.output.config import Encoder, RenderConfig, Zoom


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
        lambda: RenderConfig(max_tracks=-1),
        lambda: RenderConfig(zoom=cast("Zoom", "some")),
        lambda: RenderConfig(zoom_below_area=0),
        lambda: RenderConfig(zoom_below_amplitude=-1),
    ],
)
def test_invalid_render_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match="must be"):
        build()


def test_videos_are_rendered_up_to_max_tracks_and_skipped_above() -> None:
    cfg = RenderConfig(max_tracks=300)

    assert cfg.renders_videos_for(300)
    assert not cfg.renders_videos_for(301)


def test_zero_max_tracks_renders_videos_whatever_the_track_count() -> None:
    assert RenderConfig(max_tracks=0).renders_videos_for(100_000)
