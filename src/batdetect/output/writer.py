from __future__ import annotations

import contextlib
import subprocess
from pathlib import Path
from types import TracebackType
from typing import IO, Self

from batdetect.output.config import RenderConfig
from batdetect.output.encoder import encoder_args
from batdetect.video import VideoError, VideoInfo


class FrameWriter:
    def __init__(self, path: Path, info: VideoInfo, cfg: RenderConfig, slowdown: int = 1) -> None:
        command = [
            "ffmpeg",
            "-v",
            "error",
            "-y",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "bgr24",
            "-s",
            f"{info.width}x{info.height}",
            "-r",
            str(info.fps / slowdown),
            "-i",
            "-",
            *encoder_args(cfg),
            str(path),
        ]
        self.path = path
        self._process = subprocess.Popen(command, stdin=subprocess.PIPE)
        stdin = self._process.stdin
        if stdin is None:
            self._process.kill()
            self._process.wait()
            raise VideoError("ffmpeg stdin unavailable")
        self._stdin: IO[bytes] = stdin
        self._finished = False

    @property
    def running(self) -> bool:
        return self._process.poll() is None

    def write(self, frame: bytes) -> None:
        try:
            self._stdin.write(frame)
        except BrokenPipeError as err:
            raise VideoError(f"ffmpeg failed while writing {self.path}") from err

    def close(self) -> None:
        if self._finished:
            return
        with contextlib.suppress(BrokenPipeError):
            self._stdin.close()
        code = self._process.wait()
        self._finished = True
        if code != 0:
            self.path.unlink(missing_ok=True)
            raise VideoError(f"ffmpeg failed while writing {self.path}")

    def abort(self) -> None:
        if self._finished:
            return
        self._finished = True
        self._process.kill()
        self._process.wait()
        with contextlib.suppress(BrokenPipeError):
            self._stdin.close()
        self.path.unlink(missing_ok=True)

    def __enter__(self) -> Self:
        return self

    def __exit__(
        self, kind: type[BaseException] | None, error: BaseException | None, traceback: TracebackType | None
    ) -> None:
        if kind is None:
            self.close()
        else:
            self.abort()
