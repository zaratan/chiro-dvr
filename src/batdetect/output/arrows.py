from __future__ import annotations

import math

import numpy as np

from batdetect.output.geometry import Points, cumulative_length, end_direction

ARROW_HALF_ANGLE = math.radians(25)
END_SPAN_FRACTION = 0.03


def arrow_head(points: Points, length: float) -> Points | None:
    span = max(END_SPAN_FRACTION * float(cumulative_length(points)[-1]), length)
    direction = end_direction(points, span)
    if direction is None:
        return None
    tip = points[-1]
    angle = math.atan2(direction[1], direction[0]) + math.pi
    wings = [
        (tip[0] + length * math.cos(angle + side), tip[1] + length * math.sin(angle + side))
        for side in (-ARROW_HALF_ANGLE, ARROW_HALF_ANGLE)
    ]
    return np.array([(tip[0], tip[1]), *wings], dtype=np.float64)
