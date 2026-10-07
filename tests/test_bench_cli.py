from __future__ import annotations

import json
from pathlib import Path

import pytest

from batdetect.arguments import MODES, build_detect_config
from batdetect.bench.cli import build_parser, main
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


def test_bench_without_ffprobe_exits_with_a_usage_error(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    def no_ffprobe(_name: str) -> None:
        return None

    monkeypatch.setattr("batdetect.bench.cli.shutil.which", no_ffprobe)

    with pytest.raises(SystemExit):
        main([str(tmp_path / "v.mp4")])

    assert "ffprobe not found" in capsys.readouterr().err


def test_bench_with_zero_workers_exits_with_a_usage_error(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit) as exit_info:
        main(["v.mp4", "--workers", "0"])

    assert exit_info.value.code == 2
    assert "--workers: must be >= 1, got 0" in capsys.readouterr().err


def test_bench_with_a_background_step_too_large_for_the_frame_rate_fails_with_a_message(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    video = tmp_path / "plain.mp4"
    write_video(video, [background(320, 240, seed=i) for i in range(40)], fps=30)

    code = main([str(video), "-o", str(tmp_path / "bench"), "--bg-step", "40"])

    assert code == 1
    assert "bg_step=40 keeps fewer than 3" in capsys.readouterr().err


def test_bench_on_a_missing_video_fails_with_a_message_instead_of_a_traceback(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    missing = tmp_path / "missing.mp4"

    code = main([str(missing), "-o", str(tmp_path / "bench")])

    assert code == 1
    assert f"{missing}: [Errno 2] No such file or directory" in capsys.readouterr().err


def test_bench_measures_the_same_quick_mode_as_the_command() -> None:
    assert build_detect_config(build_parser().parse_args(["v.mp4", "--mode", "quick"])) == MODES["quick"]
