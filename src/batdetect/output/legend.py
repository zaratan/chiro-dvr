from __future__ import annotations

import math
from dataclasses import dataclass

import cv2
import numpy as np

from batdetect.output.marker import draw_marker
from batdetect.output.style import FONT, PANEL_COLOR, PANEL_TEXT, Color, SummaryStyle, text_size
from batdetect.video import ColorFrame

ELLIPSIS = "..."
WIDEST_TIME = "00:00"
HEADER_LINE_SPACING = 1.8
MAX_HEADER_COLUMNS = 3


@dataclass(frozen=True, slots=True)
class LegendEntry:
    label: str
    color: Color
    text: str


def fit_text(text: str, width: int, style: SummaryStyle) -> str:
    if text_size(text, style)[0] <= width:
        return text
    while text and text_size(text + ELLIPSIS, style)[0] > width:
        text = text[:-1]
    return text + ELLIPSIS


def column_width(style: SummaryStyle, texts: list[str]) -> int:
    widest = max(text_size(t, style)[0] for t in [WIDEST_TIME, *texts])
    return 2 * style.radius + widest + 3 * style.margin


def row_height(style: SummaryStyle) -> int:
    return 2 * style.radius + style.margin // 2


def legend_panel(header: list[str], entries: list[LegendEntry], height: int, style: SummaryStyle) -> ColorFrame:
    line_height = round(text_size(WIDEST_TIME, style)[1] * HEADER_LINE_SPACING)
    top = style.margin + line_height * len(header) + style.margin
    rows = max(1, (height - top - style.margin) // row_height(style))
    columns = max(1, math.ceil(len(entries) / rows))
    column = column_width(style, [e.text for e in entries])
    widest_header = max((text_size(text, style)[0] for text in header), default=0) + 2 * style.margin
    width = max(columns * column + style.margin, min(widest_header, MAX_HEADER_COLUMNS * column))
    panel = np.full((height, width, 3), PANEL_COLOR, dtype=np.uint8)
    for n, text in enumerate(header):
        origin = (style.margin, style.margin + line_height * (n + 1))
        line = fit_text(text, width - 2 * style.margin, style)
        cv2.putText(panel, line, origin, FONT, style.font_scale, PANEL_TEXT, style.font_thickness, cv2.LINE_AA)
    for n, entry in enumerate(entries):
        index, row = divmod(n, rows)
        cx = style.margin + index * column + style.radius
        cy = top + row * row_height(style) + style.radius
        draw_marker(panel, (cx, cy), entry.label, entry.color, style)
        origin = (cx + style.radius + style.margin, cy + text_size(entry.text, style)[1] // 2)
        cv2.putText(panel, entry.text, origin, FONT, style.font_scale, PANEL_TEXT, style.font_thickness, cv2.LINE_AA)
    return panel
