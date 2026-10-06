from __future__ import annotations

import subprocess
import tempfile
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path

from batdetect.damage import Probe, parse_probe
from batdetect.video import VideoError

PROBE_ARGS = (
    "-v",
    "error",
    "-threads",
    "1",
    "-show_log",
    "16",
    "-select_streams",
    "v:0",
    "-show_entries",
    "frame=pts,key_frame:log=message",
    "-of",
    "json",
)


def ffprobe_version() -> str:
    completed = subprocess.run(["ffprobe", "-version"], capture_output=True, check=True)
    return completed.stdout.decode(errors="replace").partition("\n")[0]


class ProbeOutcome:
    def __init__(self, process: subprocess.Popen[bytes]) -> None:
        self.process = process
        self.found: Probe | None = None

    @property
    def probe(self) -> Probe:
        if self.found is None:
            raise RuntimeError("the probe is only available after its block ends without error")
        return self.found


@contextmanager
def probing(video: Path) -> Generator[ProbeOutcome]:
    with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
        process = subprocess.Popen(
            ["ffprobe", *PROBE_ARGS, str(video)], stdout=output, stderr=errors, stdin=subprocess.DEVNULL
        )
        outcome = ProbeOutcome(process)
        try:
            yield outcome
        except BaseException:
            process.kill()
            raise
        finally:
            process.wait()
        if process.returncode != 0:
            errors.seek(0)
            last = errors.read().decode(errors="replace").strip().splitlines()[-1:]
            raise VideoError(f"ffprobe failed on {video} (exit code {process.returncode}) {' '.join(last)}".rstrip())
        output.seek(0)
        outcome.found = parse_probe(output.read().decode(errors="replace"))
