from __future__ import annotations

import argparse

import pytest

from batdetect.arguments import parse_region


def test_osd_region_is_parsed_from_four_fractions() -> None:
    region = parse_region("0,0.9,0.5,1")

    assert (region.x0, region.y0, region.x1, region.y1) == (0, 0.9, 0.5, 1)


@pytest.mark.parametrize("text", ["0,0,1", "a,b,c,d", "0.5,0,0.2,1"])
def test_malformed_osd_region_is_refused(text: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError):
        parse_region(text)
