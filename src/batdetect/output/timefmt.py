from __future__ import annotations

import math


def format_time(seconds: float) -> str:
    centis = round(seconds * 100)
    minutes, rest = divmod(centis, 6000)
    return f"{minutes}:{rest / 100:05.2f}"


def format_clock(seconds: float) -> str:
    minutes, rest = divmod(int(seconds), 60)
    return f"{minutes}:{rest:02d}"


def format_duration(seconds: float) -> str:
    minutes, rest = divmod(math.ceil(seconds), 60)
    return f"{minutes} min {rest} s" if minutes else f"{rest} s"
