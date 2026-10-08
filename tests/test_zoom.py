from __future__ import annotations

import pytest

from batdetect.detect import Detection
from batdetect.output.config import RenderConfig
from batdetect.output.zoom import MAX_MAGNIFICATION, ZOOM_MARGIN_PX, Crop, needs_zoom, zoom_crop
from batdetect.track import Track
from helpers import line

WIDTH, HEIGHT = 1440, 1080


def blob(frame: int, x: float, y: float, area: float, amplitude: float) -> Detection:
    return Detection(frame, x, y, x - 2, y - 2, 4, 4, area, amplitude)


def flight(area: float, amplitude: float) -> Track:
    return Track(1, [blob(f, 500 + 30 * f, 400, area, amplitude) for f in range(8)])


def test_faint_small_track_gets_a_zoom_but_a_bright_large_one_does_not() -> None:
    cfg = RenderConfig()

    assert needs_zoom(flight(area=36, amplitude=34.7), cfg)
    assert not needs_zoom(flight(area=338, amplitude=110.3), cfg)


def test_small_but_bright_track_gets_a_zoom() -> None:
    assert needs_zoom(flight(area=70, amplitude=110), RenderConfig())


def test_large_but_faint_track_gets_a_zoom() -> None:
    assert needs_zoom(flight(area=2583, amplitude=27.4), RenderConfig())


def test_track_exactly_at_both_thresholds_is_not_zoomed() -> None:
    cfg = RenderConfig()

    assert not needs_zoom(flight(area=cfg.zoom_below_area, amplitude=cfg.zoom_below_amplitude), cfg)


def test_zoom_all_and_none_override_the_thresholds() -> None:
    faint, bright = flight(area=36, amplitude=34.7), flight(area=338, amplitude=110.3)

    assert needs_zoom(bright, RenderConfig(zoom="all"))
    assert not needs_zoom(faint, RenderConfig(zoom="none"))


def test_short_trajectory_is_magnified_four_times_at_most() -> None:
    crop = zoom_crop(Track(1, line(0, 7, (700, 500), (1, 3))), WIDTH, HEIGHT)

    assert (crop.width, crop.height) == (WIDTH // MAX_MAGNIFICATION, HEIGHT // MAX_MAGNIFICATION)


def test_single_position_track_still_gets_the_minimum_frame() -> None:
    crop = zoom_crop(Track(1, [blob(0, 700, 500, 9, 30)]), WIDTH, HEIGHT)

    assert (crop.width, crop.height) == (360, 270)


def test_frame_covers_the_trajectory_and_its_margin() -> None:
    track = Track(1, line(0, 10, (300, 600), (60, -10)))

    crop = zoom_crop(track, WIDTH, HEIGHT)

    first, last = track.first, track.last
    assert crop.left <= first.left - ZOOM_MARGIN_PX
    assert crop.left + crop.width >= last.left + last.width + ZOOM_MARGIN_PX
    assert crop.top <= last.top - ZOOM_MARGIN_PX
    assert crop.top + crop.height >= first.top + first.height + ZOOM_MARGIN_PX


def test_tall_trajectory_widens_the_frame_to_the_source_aspect_ratio() -> None:
    crop = zoom_crop(Track(1, line(0, 10, (700, 100), (0, 70))), WIDTH, HEIGHT)

    assert crop.width * HEIGHT == crop.height * WIDTH
    assert crop.width > WIDTH // MAX_MAGNIFICATION


def test_frame_near_the_edge_is_shifted_inside_the_image() -> None:
    crop = zoom_crop(Track(1, line(0, 7, (1430, 5), (1, 2))), WIDTH, HEIGHT)

    assert crop == Crop(WIDTH - 360, 0, 360, 270)


def test_trajectory_wider_than_the_image_falls_back_to_the_whole_image() -> None:
    crop = zoom_crop(Track(1, line(0, 15, (0, 540), (100, 0))), WIDTH, HEIGHT)

    assert crop == Crop(0, 0, WIDTH, HEIGHT)


@pytest.mark.parametrize(("width", "height"), [(1440, 1080), (1920, 1080), (320, 240), (1442, 1081)])
@pytest.mark.parametrize("start", [(0, 0), (150, 120), (1900, 1070), (60, 900)])
def test_frame_is_inside_the_image_at_its_aspect_ratio_whatever_the_size(
    width: int, height: int, start: tuple[float, float]
) -> None:
    crop = zoom_crop(Track(1, line(0, 6, start, (9, -4))), width, height)

    assert crop.width > 0
    assert crop.height > 0
    assert 0 <= crop.left <= width - crop.width
    assert 0 <= crop.top <= height - crop.height
    assert crop.width * height == crop.height * width
    assert width / crop.width <= MAX_MAGNIFICATION
