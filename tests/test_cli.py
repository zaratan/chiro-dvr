from __future__ import annotations

from pathlib import Path

import pytest

from batdetect.cli import build_configs, build_parser, main
from batdetect.detect import Region


def test_options_become_validated_configs() -> None:
    ns = build_parser().parse_args(
        [
            "in",
            "--threshold",
            "30",
            "--osd-region",
            "0,0,1,0.1",
            "--max-gap",
            "8",
            "--crf",
            "28",
            "--encoder",
            "x264",
            "--vt-quality",
            "40",
        ]
    )

    detect, track, render = build_configs(ns)

    assert detect.threshold == 30
    assert detect.osd_regions == (Region(0, 0, 1, 0.1),)
    assert track.max_gap == 8
    assert render.crf == 28
    assert render.encoder == "x264"
    assert render.vt_quality == 40


def test_invalid_option_value_exits_with_a_usage_error(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit) as exit_info:
        main(["in", "--threshold", "0"])

    assert exit_info.value.code == 2
    assert "threshold must be > 0" in capsys.readouterr().err


def test_folder_without_video_exits_with_a_usage_error(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit) as exit_info:
        main([str(tmp_path)])

    assert exit_info.value.code == 2
    assert "no video found" in capsys.readouterr().err


def test_missing_ffmpeg_exits_with_a_usage_error(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    def no_ffmpeg(_name: str) -> None:
        return None

    monkeypatch.setattr("batdetect.cli.shutil.which", no_ffmpeg)

    with pytest.raises(SystemExit):
        main([str(tmp_path)])

    assert "ffmpeg not found" in capsys.readouterr().err


def test_unknown_encoder_exits_with_a_usage_error(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit) as exit_info:
        main(["in", "--encoder", "hevc"])

    assert exit_info.value.code == 2
    assert "invalid choice" in capsys.readouterr().err


def test_detection_runs_in_one_process_by_default_since_extra_chunks_only_add_re_decoding() -> None:
    assert build_parser().parse_args(["in"]).workers == 1
