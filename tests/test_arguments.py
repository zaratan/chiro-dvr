from __future__ import annotations

import argparse

import pytest

from batdetect.arguments import (
    MODES,
    add_detection_arguments,
    add_tracking_arguments,
    build_detect_config,
    parse_region,
)
from batdetect.detect import DetectConfig
from helpers import option_help


def test_osd_region_is_parsed_from_four_fractions() -> None:
    region = parse_region("0,0.9,0.5,1")

    assert (region.x0, region.y0, region.x1, region.y1) == (0, 0.9, 0.5, 1)


@pytest.mark.parametrize("text", ["0,0,1", "a,b,c,d", "0.5,0,0.2,1"])
def test_malformed_osd_region_is_refused(text: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError):
        parse_region(text)


def test_noise_factor_reaches_the_detection_config_and_zero_turns_it_off() -> None:
    parser = argparse.ArgumentParser()
    add_detection_arguments(parser)

    assert build_detect_config(parser.parse_args([])).noise_factor == DetectConfig().noise_factor
    assert build_detect_config(parser.parse_args(["--noise-factor", "5"])).noise_factor == 5
    assert build_detect_config(parser.parse_args(["--noise-factor", "0"])).noise_factor == 0


def test_target_sigma_reaches_the_detection_config_and_zero_turns_it_off() -> None:
    parser = argparse.ArgumentParser()
    add_detection_arguments(parser)

    assert build_detect_config(parser.parse_args([])).target_sigma == DetectConfig().target_sigma
    assert build_detect_config(parser.parse_args(["--target-sigma", "2"])).target_sigma == 2
    assert build_detect_config(parser.parse_args(["--target-sigma", "0"])).target_sigma == 0


def detect_config_from(argv: list[str]) -> DetectConfig:
    parser = argparse.ArgumentParser()
    add_detection_arguments(parser)
    return build_detect_config(parser.parse_args(argv))


def test_normal_mode_is_the_default_and_keeps_the_current_defaults() -> None:
    assert detect_config_from([]) == DetectConfig()
    assert detect_config_from(["--mode", "normal"]) == DetectConfig()


def test_quick_mode_sets_only_the_four_settings_used_before_the_noise_threshold() -> None:
    expected = DetectConfig(work_width=480, threshold=25, target_sigma=0, noise_factor=0)

    assert detect_config_from(["--mode", "quick"]) == expected


def test_explicit_option_wins_over_the_mode_whatever_their_order() -> None:
    assert detect_config_from(["--threshold", "30", "--mode", "quick"]).threshold == 30
    assert detect_config_from(["--mode", "quick", "--work-width", "960"]).work_width == 960
    assert detect_config_from(["--mode", "quick", "--noise-factor", "8"]).noise_factor == 8
    assert detect_config_from(["--mode", "quick", "--target-sigma", "1.5"]).target_sigma == 1.5


def test_unknown_mode_is_refused() -> None:
    with pytest.raises(SystemExit):
        detect_config_from(["--mode", "fine"])


def shared_option_help() -> dict[str, str]:
    parser = argparse.ArgumentParser()
    add_detection_arguments(parser)
    add_tracking_arguments(parser)
    return option_help(parser.format_help())


def test_options_set_by_the_mode_show_the_value_of_each_mode_read_from_the_modes_table() -> None:
    entries = shared_option_help()
    normal, quick = MODES["normal"], MODES["quick"]

    assert f"(normal: {normal.threshold:g}, quick: {quick.threshold:g})" in entries["--threshold"]
    assert f"(normal: {normal.work_width}, quick: {quick.work_width})" in entries["--work-width"]
    assert f"(normal: {normal.noise_factor:g}, quick: {quick.noise_factor:g})" in entries["--noise-factor"]
    assert f"(normal: {normal.target_sigma:g}, quick: {quick.target_sigma:g})" in entries["--target-sigma"]


def test_shared_detection_and_tracking_options_show_their_default_for_the_bench_too() -> None:
    entries = shared_option_help()
    silent = [
        option
        for option, text in entries.items()
        if option not in {"-h", "--threshold", "--work-width", "--noise-factor", "--target-sigma"}
        and "(default: " not in text
    ]

    assert silent == []
    assert "(default: 6)" in entries["--max-gap"]
