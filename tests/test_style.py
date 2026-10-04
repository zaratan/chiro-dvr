from __future__ import annotations

import itertools
import math

from batdetect.output.style import MIN_RADIUS, PALETTE, Color, style_for

MIN_LUMINANCE = 0.25
MIN_COLOR_DISTANCE = 45


def luminance(color: Color) -> float:
    def linear(channel: int) -> float:
        c = channel / 255
        return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4

    blue, green, red = color
    return 0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue)


def test_sizes_grow_with_the_video_width() -> None:
    small, large = style_for(720), style_for(1440)

    assert large.radius > small.radius
    assert large.arrow_length == 2 * small.arrow_length
    assert large.line >= small.line


def test_small_video_keeps_readable_minimum_sizes() -> None:
    style = style_for(320)

    assert style.radius >= MIN_RADIUS
    assert style.line >= 1
    assert style.font_thickness >= 1


def test_outline_is_wider_than_the_line_it_surrounds() -> None:
    for width in (320, 1440, 3840):
        style = style_for(width)
        assert style.outline > style.line


def test_marker_grows_to_hold_a_three_digit_number() -> None:
    assert style_for(1440, "999").radius > style_for(1440, "9").radius


def test_every_palette_color_stays_bright_against_black_outlines() -> None:
    assert all(luminance(c) >= MIN_LUMINANCE for c in PALETTE)


def test_palette_colors_are_pairwise_distinct() -> None:
    assert min(math.dist(a, b) for a, b in itertools.combinations(PALETTE, 2)) >= MIN_COLOR_DISTANCE
