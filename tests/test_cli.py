from __future__ import annotations

import tomllib
from pathlib import Path

import pytest

from batdetect.cli import build_configs, build_parser, main
from batdetect.detect import Region
from helpers import ANSI_STYLE, option_help

MODE_OPTIONS = ("--threshold", "--work-width", "--noise-factor", "--target-sigma")
PYPROJECT = Path(__file__).parents[1] / "pyproject.toml"


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
            "--zoom",
            "all",
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
    assert render.zoom == "all"
    assert (settings.stability.max_blobs, settings.stability.pad_s) == (0, 2.5)


def test_mode_is_kept_in_the_settings_for_params_json() -> None:
    settings = build_configs(build_parser().parse_args(["in", "--mode", "quick"]))

    assert settings.mode == "quick"
    assert settings.detect.work_width == 480


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


def test_version_matches_pyproject_so_the_release_can_compare_it_to_the_tag(
    capsys: pytest.CaptureFixture[str],
) -> None:
    with pytest.raises(SystemExit) as exit_info:
        main(["--version"])

    assert exit_info.value.code == 0
    project = tomllib.loads(PYPROJECT.read_text())["project"]
    assert capsys.readouterr().out == f"batdetect {project['version']}\n"


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


def test_max_tracks_defaults_between_the_busiest_healthy_video_and_the_smallest_known_explosion() -> None:
    assert build_configs(build_parser().parse_args(["in"])).render.max_tracks == 300


def test_max_tracks_option_reaches_the_render_config_so_zero_can_disable_it() -> None:
    assert build_configs(build_parser().parse_args(["in", "--max-tracks", "0"])).render.max_tracks == 0


def test_zoom_is_kept_for_small_or_faint_tracks_by_default() -> None:
    assert build_configs(build_parser().parse_args(["in"])).render.zoom == "auto"


def test_unknown_zoom_exits_with_a_usage_error(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit) as exit_info:
        main(["in", "--zoom", "some"])

    assert exit_info.value.code == 2
    assert "--zoom" in capsys.readouterr().err


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


def test_every_option_describes_its_effect_and_shows_its_default_so_a_setting_can_be_changed_knowingly() -> None:
    entries = option_help(build_parser().format_help())
    undocumented = [
        option
        for option, text in entries.items()
        if option not in {"-h", "--version", *MODE_OPTIONS} and "(default: " not in text
    ]

    assert len(entries) == 32
    assert undocumented == []


def test_value_names_say_what_the_option_expects_rather_than_repeat_its_name() -> None:
    usage = ANSI_STYLE.sub("", build_parser().format_usage())

    assert "-o DIR" in usage
    assert "--trail SECONDS" in usage
    assert "TRAIL" not in usage


def test_every_help_text_is_translated_and_never_wrapped_before_a_colon_under_a_french_locale(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    english = option_help(build_parser().format_help())
    monkeypatch.setenv("LANG", "fr_FR.UTF-8")
    french_help = build_parser().format_help()
    french = option_help(french_help)

    assert french.keys() == english.keys()
    assert [option for option in english if french[option] == english[option]] == []
    assert french_help.startswith("utilisation : batdetect")
    assert "(défaut : 1,5)" in french["--clip-margin"]
    assert "(défaut\N{NO-BREAK SPACE}:\N{NO-BREAK SPACE}1,5)" in french_help


@pytest.mark.usefixtures("french")
def test_missing_inputs_are_reported_in_french_by_argparse_itself(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit):
        main([])

    err = capsys.readouterr().err
    assert err.startswith("utilisation : batdetect")
    assert "batdetect : erreur : arguments obligatoires manquants : inputs" in err


@pytest.mark.parametrize("lang", ["C", "en_US.UTF-8"])
def test_missing_inputs_stay_in_english_under_a_non_french_locale(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str], lang: str
) -> None:
    monkeypatch.setenv("LANG", lang)
    with pytest.raises(SystemExit):
        main([])

    assert "batdetect: error: the following arguments are required: inputs" in capsys.readouterr().err


@pytest.mark.usefixtures("french")
def test_our_own_usage_errors_are_in_french_too(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit):
        main(["in", "--workers", "0"])

    assert "argument --workers : doit être >= 1, reçu 0" in capsys.readouterr().err
