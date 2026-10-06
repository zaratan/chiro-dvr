from __future__ import annotations

import argparse

import pytest

from batdetect.arguments import add_detection_arguments, build_detect_config, parse_region
from batdetect.detect import DetectConfig


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
