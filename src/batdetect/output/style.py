from __future__ import annotations

import math
from dataclasses import dataclass

import cv2

Color = tuple[int, int, int]

FONT = cv2.FONT_HERSHEY_DUPLEX
PALETTE: tuple[Color, ...] = (
    (10, 214, 255),
    (240, 201, 76),
    (26, 140, 255),
    (91, 227, 91),
    (203, 92, 255),
    (255, 168, 110),
    (60, 240, 200),
    (255, 140, 185),
    (92, 92, 255),
    (235, 235, 235),
    (140, 200, 255),
    (170, 170, 20),
)
INK: Color = (0, 0, 0)
PANEL_COLOR: Color = (32, 32, 32)
PANEL_TEXT: Color = (230, 230, 230)
SEPARATOR: Color = (80, 80, 80)
BACKGROUND_DIM = 0.6
MIN_RADIUS = 8
MIN_TEXT_PX = 8
MIN_ARROW = 8
TEXT_TO_RADIUS = 0.9


@dataclass(frozen=True, slots=True)
class SummaryStyle:
    line: int
    outline: int
    radius: int
    border: int
    arrow_length: int
    font_scale: float
    font_thickness: int
    margin: int
    neighbor_distance: float


def text_size(text: str, style: SummaryStyle) -> tuple[int, int]:
    (width, height), _ = cv2.getTextSize(text, FONT, style.font_scale, style.font_thickness)
    return int(width), int(height)


def style_for(width: int, widest_label: str = "99") -> SummaryStyle:
    line = max(1, round(width / 480))
    border = max(1, round(width / 720))
    base_radius = max(MIN_RADIUS, round(width / 64))
    text_px = max(MIN_TEXT_PX, round(base_radius * TEXT_TO_RADIUS))
    font_scale = float(cv2.getFontScaleFromHeight(FONT, text_px, border))
    (label_width, _), _ = cv2.getTextSize(widest_label, FONT, font_scale, border)
    return SummaryStyle(
        line=line,
        outline=line + 2 * max(1, round(width / 960)),
        radius=max(base_radius, math.ceil(label_width / 2) + 2 * border),
        border=border,
        arrow_length=max(MIN_ARROW, round(width / 48)),
        font_scale=font_scale,
        font_thickness=border,
        margin=max(4, round(width / 90)),
        neighbor_distance=width / 6,
    )
