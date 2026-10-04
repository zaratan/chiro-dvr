from __future__ import annotations

import cv2

from batdetect.output.style import FONT, INK, Color, SummaryStyle, text_size
from batdetect.video import ColorFrame


def draw_marker(image: ColorFrame, center: tuple[float, float], label: str, color: Color, style: SummaryStyle) -> None:
    cx, cy = round(center[0]), round(center[1])
    cv2.circle(image, (cx, cy), style.radius, color, -1, cv2.LINE_AA)
    cv2.circle(image, (cx, cy), style.radius, INK, style.border, cv2.LINE_AA)
    width, height = text_size(label, style)
    origin = (cx - width // 2, cy + height // 2)
    cv2.putText(image, label, origin, FONT, style.font_scale, INK, style.font_thickness, cv2.LINE_AA)
