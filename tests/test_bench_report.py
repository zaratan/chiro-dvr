from __future__ import annotations

from pathlib import Path

import pytest

from batdetect.bench.metrics import Evaluation
from batdetect.bench.report import code_version, format_table


def test_table_has_a_line_per_class_and_a_totals_line() -> None:
    row: dict[str, object] = {
        "amplitude": -60.0,
        "sigma": 3.0,
        "n": 26,
        "found": 24,
        "found_ci95": [0.759, 0.979],
        "median_completeness": 0.95,
        "median_start_delay": 0,
        "median_end_early": 1,
        "mean_fragments": 1.0,
        "median_effective": -45.8,
    }

    table = format_table(
        [row, row], Evaluation([], not_visible=17, false_tracks=0, reference_tracks=14, injected_tracks=136)
    )

    lines = table.splitlines()
    assert len(lines) == 4
    assert "-60.0" in lines[1]
    assert lines[-1] == "not visible: 17  false tracks: 0  tracks: 136 injected run, 14 reference"


def test_code_version_is_unknown_outside_a_git_repository(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.chdir(tmp_path)

    assert code_version() == "unknown"
