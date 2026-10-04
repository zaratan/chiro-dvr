from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

from batdetect.output.config import RenderConfig
from batdetect.output.encoder import encoder_args
from batdetect.output.timefmt import clip_name
from batdetect.track import Track
from batdetect.video import VideoError, VideoInfo


def split_clips(annotated: Path, tracks: list[Track], info: VideoInfo, split_dir: Path, cfg: RenderConfig) -> None:
    shutil.rmtree(split_dir, ignore_errors=True)
    split_dir.mkdir(parents=True)
    duration = info.frame_count / info.fps
    for track in tracks:
        start = max(0.0, track.first.frame / info.fps - cfg.clip_margin_s)
        end = min(duration, track.last.frame / info.fps + cfg.clip_margin_s)
        command = [
            "ffmpeg",
            "-v",
            "error",
            "-y",
            "-ss",
            f"{start:.3f}",
            "-i",
            str(annotated),
            "-t",
            f"{end - start:.3f}",
            *encoder_args(cfg),
            str(split_dir / clip_name(track, info.fps)),
        ]
        if subprocess.run(command, check=False).returncode != 0:
            raise VideoError(f"ffmpeg failed while cutting clip #{track.id}")
