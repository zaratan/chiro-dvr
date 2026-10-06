from __future__ import annotations

from collections.abc import Sequence

import numpy as np
import numpy.typing as npt

from batdetect.median import NumericFrame, temporal_median

MAD_TO_SIGMA = 1.4826


def temporal_noise(
    samples: Sequence[NumericFrame], background: npt.NDArray[np.float64], gains: Sequence[float]
) -> npt.NDArray[np.float64]:
    deviations = [(s - background - g).astype(np.float32) for s, g in zip(samples, gains, strict=True)]
    center = temporal_median(deviations).astype(np.float32)
    return MAD_TO_SIGMA * temporal_median([np.abs(d - center) for d in deviations])
