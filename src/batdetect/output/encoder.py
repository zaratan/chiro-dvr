from __future__ import annotations

import subprocess
from collections.abc import Callable
from dataclasses import replace

from batdetect.output.config import AUTO, VIDEOTOOLBOX, X264, RenderConfig

PROBE_INPUT = ["-f", "lavfi", "-i", "color=c=gray:s=64x64:r=30:d=0.1"]


def encoder_args(cfg: RenderConfig) -> list[str]:
    if cfg.encoder == VIDEOTOOLBOX:
        codec = ["-c:v", "h264_videotoolbox", "-q:v", str(cfg.vt_quality)]
    elif cfg.encoder == X264:
        codec = ["-c:v", "libx264", "-crf", str(cfg.crf)]
    else:
        raise ValueError("encoder must be resolved before rendering")
    return [*codec, "-pix_fmt", "yuv420p"]


def encoder_failure(cfg: RenderConfig) -> str | None:
    command = ["ffmpeg", "-v", "error", *PROBE_INPUT, *encoder_args(cfg), "-f", "null", "-"]
    try:
        result = subprocess.run(command, capture_output=True, text=True, check=False)
    except FileNotFoundError:
        return "ffmpeg not found"
    if result.returncode == 0:
        return None
    return result.stderr.strip() or f"ffmpeg exited with code {result.returncode}"


def resolve_encoder(cfg: RenderConfig, probe: Callable[[RenderConfig], str | None] = encoder_failure) -> RenderConfig:
    if cfg.encoder == AUTO:
        hardware = replace(cfg, encoder=VIDEOTOOLBOX)
        return hardware if probe(hardware) is None else replace(cfg, encoder=X264)
    failure = probe(cfg)
    if failure is not None:
        raise ValueError(f"encoder {cfg.encoder} is not usable with this ffmpeg: {failure}")
    return cfg
