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
            "--annotated",
            "--max-blobs",
            "0",
            "--unstable-pad",
            "2.5",
            "--max-median-turn",
            "1.2",
        ]
    )

    settings = build_configs(ns)
    detect, track, render = settings.detect, settings.track, settings.render

    assert detect.threshold == 30
    assert detect.osd_regions == (Region(0, 0, 1, 0.1),)
    assert track.max_gap == 8
    assert track.max_median_turn == 1.2
    assert render.crf == 28
    assert render.encoder == "x264"
    assert render.vt_quality == 40
    assert render.annotated
    assert (settings.stability.max_blobs, settings.stability.pad_s) == (0, 2.5)


def test_invalid_option_value_exits_with_a_usage_error(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit) as exit_info:
        main(["in", "--threshold", "0"])

    assert exit_info.value.code == 2
    assert "threshold must be > 0" in capsys.readouterr().err


def test_zero_workers_exits_with_a_usage_error_instead_of_silently_running_one(
    capsys: pytest.CaptureFixture[str],
) -> None:
    with pytest.raises(SystemExit) as exit_info:
        main(["in", "--workers", "0"])

    assert exit_info.value.code == 2
    assert "--workers: must be >= 1, got 0" in capsys.readouterr().err


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


def test_whole_annotated_video_is_off_by_default() -> None:
    assert not build_configs(build_parser().parse_args(["in"])).render.annotated


def test_missing_ffprobe_exits_with_a_usage_error(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    def only_ffmpeg(name: str) -> str | None:
        return "/usr/bin/ffmpeg" if name == "ffmpeg" else None

    monkeypatch.setattr("batdetect.cli.shutil.which", only_ffmpeg)

    with pytest.raises(SystemExit):
        main([str(tmp_path)])

    assert "ffprobe not found" in capsys.readouterr().err
