from __future__ import annotations

import math
import statistics
from collections.abc import Sequence

from batdetect.bench.metrics import BatResult, Evaluation
from batdetect.synthetic.trajectory import MOTIONS

WILSON_Z = 1.96


def wilson(successes: int, total: int) -> tuple[float, float]:
    if total == 0:
        return 0.0, 0.0
    p = successes / total
    denom = 1 + WILSON_Z**2 / total
    center = (p + WILSON_Z**2 / (2 * total)) / denom
    half = WILSON_Z * math.sqrt(p * (1 - p) / total + WILSON_Z**2 / (4 * total**2)) / denom
    return max(0.0, center - half), min(1.0, center + half)


def _median_or_none(values: Sequence[float]) -> float | None:
    return statistics.median(values) if values else None


def summarize(evaluation: Evaluation) -> list[dict[str, object]]:
    groups: dict[tuple[str, float, float], list[BatResult]] = {}
    for result in evaluation.bats:
        groups.setdefault((result.motion, result.amplitude, result.sigma), []).append(result)
    rows: list[dict[str, object]] = []
    for (motion, amplitude, sigma), results in sorted(groups.items(), key=lambda g: (MOTIONS.index(g[0][0]), g[0][1:])):
        found = [r for r in results if r.found]
        low, high = wilson(len(found), len(results))
        rows.append(
            {
                "motion": motion,
                "amplitude": amplitude,
                "sigma": sigma,
                "n": len(results),
                "found": len(found),
                "found_ci95": [round(low, 3), round(high, 3)],
                "median_completeness": round(statistics.median(r.completeness for r in results), 3),
                "median_start_delay": _median_or_none([r.start_delay for r in found if r.start_delay is not None]),
                "median_end_early": _median_or_none([r.end_early for r in found if r.end_early is not None]),
                "mean_fragments": round(statistics.mean(r.fragments for r in found), 2) if found else None,
                "median_effective": round(statistics.median(r.median_effective for r in results), 1),
            }
        )
    return rows
