from __future__ import annotations

from collections.abc import Sequence
from typing import Any

import numpy as np
import numpy.typing as npt

IntFrame = npt.NDArray[np.integer[Any]]
NumericFrame = npt.NDArray[np.integer[Any] | np.floating[Any]]


def temporal_median(frames: Sequence[NumericFrame]) -> npt.NDArray[np.float64]:
    n = len(frames)
    if n == 0:
        raise ValueError("temporal_median needs at least one frame")
    values = list(frames)
    lo, hi = (n - 1) // 2, n // 2
    for done in range(n - lo):
        for j in range(n - 1 - done):
            values[j], values[j + 1] = np.minimum(values[j], values[j + 1]), np.maximum(values[j], values[j + 1])
    return (values[lo].astype(np.float64) + values[hi]) / 2
