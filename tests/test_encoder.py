from __future__ import annotations

import pytest

from batdetect.output.config import AUTO, VIDEOTOOLBOX, X264, RenderConfig
from batdetect.output.encoder import encoder_args, encoder_works, resolve_encoder
from helpers import requires_ffmpeg


def always(_cfg: RenderConfig) -> bool:
    return True


def never(_cfg: RenderConfig) -> bool:
    return False


def only_x264(cfg: RenderConfig) -> bool:
    return cfg.encoder == X264


def test_videotoolbox_uses_its_quality_scale() -> None:
    args = encoder_args(RenderConfig(encoder=VIDEOTOOLBOX, vt_quality=70))

    assert args == ["-c:v", "h264_videotoolbox", "-q:v", "70", "-pix_fmt", "yuv420p"]


def test_x264_uses_crf() -> None:
    assert encoder_args(RenderConfig(encoder=X264, crf=23)) == ["-c:v", "libx264", "-crf", "23", "-pix_fmt", "yuv420p"]


def test_unresolved_encoder_cannot_render() -> None:
    with pytest.raises(ValueError, match="resolved"):
        encoder_args(RenderConfig(encoder=AUTO))


def test_auto_prefers_the_media_engine_when_it_works() -> None:
    assert resolve_encoder(RenderConfig(), always).encoder == VIDEOTOOLBOX


def test_auto_falls_back_to_x264_where_the_media_engine_is_missing() -> None:
    assert resolve_encoder(RenderConfig(), only_x264).encoder == X264


def test_explicit_encoder_that_does_not_work_is_refused_up_front() -> None:
    with pytest.raises(ValueError, match="videotoolbox is not usable"):
        resolve_encoder(RenderConfig(encoder=VIDEOTOOLBOX), never)


def test_explicit_encoder_that_works_is_kept() -> None:
    cfg = RenderConfig(encoder=X264, crf=18)

    assert resolve_encoder(cfg, always) == cfg


@requires_ffmpeg
def test_x264_probe_succeeds_with_a_real_ffmpeg() -> None:
    assert encoder_works(RenderConfig(encoder=X264))
