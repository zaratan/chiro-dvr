from __future__ import annotations

import json
from pathlib import Path

from batdetect.bench.cli import main
from helpers import background, write_video


def test_bench_command_writes_a_report_on_a_synthetic_video(tmp_path: Path) -> None:
    video = tmp_path / "plain.mp4"
    write_video(video, [background(320, 240, seed=i) for i in range(150)], fps=30)
    out = tmp_path / "bench"

    code = main([str(video), "-o", str(out), "--per-class", "3", "--amplitudes", "-80", "--sigmas", "4"])

    report = json.loads((out / "plain" / "bench.json").read_text())
    assert code == 0
    assert report["summary"][0]["n"] + report["not_visible"] == 3
    assert report["summary"][0]["found"] >= 1
