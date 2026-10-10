from __future__ import annotations

import math
from pathlib import Path

import cv2
import numpy as np

from batdetect.language import _, ngettext
from batdetect.output.arrows import arrow_head
from batdetect.output.colors import assign_colors
from batdetect.output.geometry import Points, track_points
from batdetect.output.legend import LegendEntry, legend_panel
from batdetect.output.marker import draw_marker
from batdetect.output.names import summary_png
from batdetect.output.periods import Period
from batdetect.output.placement import place_markers
from batdetect.output.style import BACKGROUND_DIM, INK, PALETTE, SEPARATOR, SummaryStyle, style_for
from batdetect.output.timefmt import format_clock, format_duration
from batdetect.track import Track
from batdetect.video import ColorFrame, VideoError, VideoInfo

MAX_IGNORED_LINES = 3


def polyline(points: Points) -> list[np.ndarray[tuple[int, int], np.dtype[np.int32]]]:
    return [np.round(points).astype(np.int32)]


def ignored_lines(period: Period) -> list[str]:
    lines = [
        _("not analysed {start} - {end}").format(start=format_clock(a), end=format_clock(math.ceil(b)))
        for a, b in period.ignored
    ]
    if len(lines) > MAX_IGNORED_LINES:
        hidden = len(lines) - MAX_IGNORED_LINES
        return [*lines[:MAX_IGNORED_LINES], ngettext("+ {count} more", "+ {count} more", hidden).format(count=hidden)]
    return lines


def damaged_lines(period: Period) -> list[str]:
    if not period.damaged:
        return []
    count = len(period.damaged)
    total = sum(b - a for a, b in period.damaged)
    line = ngettext("unreadable {duration} ({count} span)", "unreadable {duration} ({count} spans)", count)
    return [line.format(duration=format_duration(total), count=count)]


def header_lines(stem: str, period: Period) -> list[str]:
    count = len(period.tracks)
    track_count = ngettext("{count} track", "{count} tracks", count).format(count=count)
    return [
        stem,
        track_count,
        f"{format_clock(period.start_s)} - {format_clock(period.end_s)}",
        *ignored_lines(period),
        *damaged_lines(period),
    ]


def compose_summary(
    background: ColorFrame, tracks: list[Track], fps: float, header: list[str], style: SummaryStyle
) -> ColorFrame:
    height, width = background.shape[:2]
    image = (background.astype(np.float64) * BACKGROUND_DIM).astype(np.uint8)
    paths = [track_points(t) for t in tracks]
    colors = [PALETTE[c] for c in assign_colors(paths, len(PALETTE), style.neighbor_distance)]
    heads = [arrow_head(p, style.arrow_length) for p in paths]
    for path in paths:
        cv2.polylines(image, polyline(path), False, INK, style.outline, cv2.LINE_AA)
    for path, color in zip(paths, colors, strict=True):
        cv2.polylines(image, polyline(path), False, color, style.line, cv2.LINE_AA)
    for head, color in zip(heads, colors, strict=True):
        if head is not None:
            cv2.fillPoly(image, polyline(head), color, cv2.LINE_AA)
            cv2.polylines(image, polyline(head), True, INK, style.border, cv2.LINE_AA)
    tips = [(float(p[-1, 0]), float(p[-1, 1])) for p, h in zip(paths, heads, strict=True) if h is not None]
    centers = place_markers(paths, (width, height), style.radius, tips, style.radius + style.arrow_length)
    entries = [
        LegendEntry(str(t.id), c, format_clock(t.first.frame / fps)) for t, c in zip(tracks, colors, strict=True)
    ]
    for entry, center in zip(entries, centers, strict=True):
        draw_marker(image, center, entry.label, entry.color, style)
    panel = legend_panel(header, entries, height, style)
    panel[:, : style.border] = SEPARATOR
    return np.hstack([image, panel])


def summary_image(background: ColorFrame, periods: list[Period], info: VideoInfo, dest: Path, stem: str) -> list[Path]:
    style = style_for(info.width, str(max((t.id for p in periods for t in p.tracks), default=0)))
    written: list[Path] = []
    for period in periods:
        image = compose_summary(background, period.tracks, info.fps, header_lines(stem, period), style)
        out = summary_png(dest, stem, period.suffix)
        if not cv2.imwrite(str(out), image):
            raise VideoError(f"cannot write {out}")
        written.append(out)
    return written
