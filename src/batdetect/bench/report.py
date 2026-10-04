from __future__ import annotations

import subprocess
from collections.abc import Sequence

from batdetect.bench.metrics import Evaluation


def code_version() -> str:
    result = subprocess.run(["git", "describe", "--always", "--dirty"], capture_output=True, text=True, check=False)
    return result.stdout.strip() or "unknown"


def format_table(rows: Sequence[dict[str, object]], evaluation: Evaluation) -> str:
    lines = [
        "motion  amplitude  sigma   n  found  ci95          completeness  start_delay  end_early  fragments  effective"
    ]
    for r in rows:
        lines.append(
            f"{r['motion']!s:<6}  {r['amplitude']:>9}  {r['sigma']:>5}  {r['n']:>2}  {r['found']:>5}  "
            f"{r['found_ci95']!s:<12}  "
            f"{r['median_completeness']!s:>12}  {r['median_start_delay']!s:>11}  {r['median_end_early']!s:>9}  "
            f"{r['mean_fragments']!s:>9}  {r['median_effective']!s:>9}"
        )
    lines.append(
        f"not visible: {evaluation.not_visible}  false tracks: {evaluation.false_tracks}  "
        f"tracks: {evaluation.injected_tracks} injected run, {evaluation.reference_tracks} reference"
    )
    return "\n".join(lines)
