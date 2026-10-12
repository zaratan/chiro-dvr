from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any

from reference_v011 import check_reference_code

from batdetect.cli import seconds
from batdetect.detect import Detection
from batdetect.output.colors import assign_colors
from batdetect.output.geometry import track_points
from batdetect.output.periods import Period, split_by_period
from batdetect.output.style import PALETTE, style_for
from batdetect.output.summary import MAX_IGNORED_LINES
from batdetect.output.timefmt import format_clock, format_duration
from batdetect.spans import Span
from batdetect.track import Track
from batdetect.video import VideoInfo


def read_tracks(path: Path) -> list[Track]:
    tracks: list[Track] = []
    with path.open(encoding="utf-8") as lines:
        for line in lines:
            entry = json.loads(line)
            tracks.append(Track(entry["id"], [Detection(**point) for point in entry["points"]]))
    return tracks


def header(stem: str, period: Period) -> list[dict[str, Any]]:
    lines: list[dict[str, Any]] = [
        {"line": "name", "text": stem},
        {"line": "tracks", "count": len(period.tracks)},
        {"line": "window", "start": format_clock(period.start_s), "end": format_clock(period.end_s)},
    ]
    for a, b in period.ignored[:MAX_IGNORED_LINES]:
        lines.append({"line": "not_analysed", "start": format_clock(a), "end": format_clock(math.ceil(b))})
    if len(period.ignored) > MAX_IGNORED_LINES:
        lines.append({"line": "more", "count": len(period.ignored) - MAX_IGNORED_LINES})
    if period.damaged:
        total = sum(b - a for a, b in period.damaged)
        lines.append({"line": "unreadable", "duration": format_duration(total), "spans": len(period.damaged)})
    return lines


def summary(recording: Path) -> dict[str, Any]:
    video = json.loads((recording / "video.json").read_text(encoding="utf-8"))
    opencv = video["opencv"]
    info = VideoInfo(
        opencv["fps"],
        opencv["frame_count_property"],
        opencv["width"],
        opencv["height"],
        opencv["work_width"],
        opencv["work_height"],
    )
    stem = Path(video["video"]["name"]).stem
    spans = {
        key: [seconds(Span(*span), info.fps) for span in video[key]] for key in ("unstable_spans", "damaged_spans")
    }
    periods = split_by_period(
        read_tracks(recording / "tracks.jsonl"),
        info,
        ignored=spans["unstable_spans"],
        damaged=spans["damaged_spans"],
    )
    style = style_for(info.width)
    result: list[dict[str, Any]] = []
    for period in periods:
        paths = [track_points(track) for track in period.tracks]
        colors = assign_colors(paths, len(PALETTE), style.neighbor_distance)
        result.append(
            {
                "suffix": period.suffix,
                "header": header(stem, period),
                "tracks": [
                    {"id": track.id, "color": color, "start": format_clock(track.first.frame / info.fps)}
                    for track, color in zip(period.tracks, colors, strict=True)
                ],
            }
        )
    return {"video": video["video"]["name"], "periods": result}


def main() -> None:
    ap = argparse.ArgumentParser(description="Record the summary layout of batdetect v0.1.1 from a recorded reference.")
    ap.add_argument("recording", type=Path, help="folder with video.json and tracks.jsonl")
    ap.add_argument("dest", type=Path, help="JSON file to write")
    args = ap.parse_args()
    check_reference_code()
    args.dest.write_text(json.dumps(summary(args.recording), ensure_ascii=False, indent=1) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
