from __future__ import annotations

import csv
import json
from dataclasses import asdict
from pathlib import Path
from typing import TYPE_CHECKING, Any

from batdetect.output.timefmt import format_time
from batdetect.track import Track
from batdetect.video import VideoInfo

if TYPE_CHECKING:
    from _typeshed import DataclassInstance


def write_tracks_csv(tracks: list[Track], info: VideoInfo, out_path: Path) -> None:
    with out_path.open("w", newline="") as fh:
        writer = csv.writer(fh)
        writer.writerow(
            [
                "id",
                "start",
                "end",
                "start_s",
                "duration_s",
                "hits",
                "chord_px",
                "path_px",
                "speed_px_s",
                "max_area_px",
                "peak_amplitude",
                "filled_frames",
            ]
        )
        for t in tracks:
            start = t.first.frame / info.fps
            duration = max((t.last.frame - t.first.frame) / info.fps, 1 / info.fps)
            path_px = t.path_length()
            writer.writerow(
                [
                    t.id,
                    format_time(start),
                    format_time(t.last.frame / info.fps),
                    round(start, 2),
                    round(duration, 2),
                    len(t.points),
                    round(t.chord(), 1),
                    round(path_px, 1),
                    round(path_px / duration, 1),
                    round(t.max_area()),
                    round(t.peak_amplitude(), 1),
                    t.last.frame - t.first.frame + 1 - len(t.points),
                ]
            )


def write_params(params: dict[str, Any], out_path: Path) -> None:
    out_path.write_text(json.dumps(params, indent=2, ensure_ascii=False) + "\n")


def config_params(*configs: DataclassInstance) -> dict[str, Any]:
    return {type(c).__name__: asdict(c) for c in configs}
