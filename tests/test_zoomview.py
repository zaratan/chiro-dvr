from __future__ import annotations

import math
from pathlib import Path

import numpy as np
import pytest

from batdetect.detect import Detection
from batdetect.output.clips import ClipWindow
from batdetect.output.config import RenderConfig
from batdetect.output.zoom import SLOW_MOTION, Crop
from batdetect.output.zoomview import TRAIL_GAP_PX, Point, Segment, trail_segments, zoom_view, zoom_windows
from batdetect.track import Track
from batdetect.video import VideoInfo
from helpers import line

INFO = VideoInfo(fps=30.0, frame_count=60, width=320, height=240, work_width=320, work_height=240)
QUARTER = Crop(100, 80, 80, 60)
WHOLE = Crop(0, 0, 320, 240)
TRAIL_FRAMES = 30


def at(frame: int, x: float, y: float, width: float = 3, height: float = 3) -> Detection:
    return Detection(frame, x, y, x - width / 2, y - height / 2, width, height, width * height, 30.0)


def textured_frame() -> np.ndarray:
    rng = np.random.default_rng(7)
    return rng.integers(0, 120, (240, 320, 3), dtype=np.uint8)


def distance_to_box(point: Point, box: Detection) -> float:
    dx = max(box.left - point[0], 0, point[0] - (box.left + box.width))
    dy = max(box.top - point[1], 0, point[1] - (box.top + box.height))
    return math.hypot(dx, dy)


def closest_approach(segments: list[Segment], box: Detection) -> float:
    samples = [
        (a[0] + (b[0] - a[0]) * t / 100, a[1] + (b[1] - a[1]) * t / 100) for a, b in segments for t in range(101)
    ]
    return min(distance_to_box(p, box) for p in samples)


@pytest.mark.parametrize("crop", [QUARTER, WHOLE])
def test_zoom_view_leaves_the_raw_frame_untouched(crop: Crop) -> None:
    track = Track(1, line(0, 10, (110, 100), (4, 1)))
    frame = textured_frame()
    before = frame.copy()

    zoom_view(track, crop, INFO, RenderConfig())(frame, 6)

    assert np.array_equal(frame, before)


def test_zoomed_frame_is_the_crop_with_each_source_pixel_made_a_square() -> None:
    track = Track(1, line(20, 10, (110, 100), (4, 1)))
    frame = textured_frame()

    zoomed = zoom_view(track, QUARTER, INFO, RenderConfig())(frame, 0)

    crop = frame[80:140, 100:180]
    assert np.array_equal(zoomed, np.repeat(np.repeat(crop, 4, axis=0), 4, axis=1))


def test_trail_appears_from_the_second_detection_and_not_before() -> None:
    track = Track(1, line(5, 10, (105, 100), (30, 0)))
    frame = textured_frame()
    view = zoom_view(track, QUARTER, INFO, RenderConfig())

    plain = zoom_view(Track(1, line(50, 6, (0, 0), (1, 0))), QUARTER, INFO, RenderConfig())(frame, 5)

    assert np.array_equal(view(frame, 4), plain)
    assert np.array_equal(view(frame, 5), plain)
    assert not np.array_equal(view(frame, 6), plain)


def test_trail_stops_short_of_the_current_position_even_at_high_speed() -> None:
    track = Track(1, line(0, 6, (20, 100), (52, 0)))

    segments = trail_segments(track, track.last, TRAIL_FRAMES)

    assert closest_approach(segments, track.last) == pytest.approx(TRAIL_GAP_PX, abs=0.5)


def test_trail_never_crosses_the_target_when_the_track_loops_back() -> None:
    points = [at(0, 100, 100), at(1, 160, 100), at(2, 210, 100), at(3, 160, 102), at(4, 100, 104)]
    track = Track(1, [*points, at(5, 160, 101)])

    segments = trail_segments(track, track.last, TRAIL_FRAMES)

    assert segments
    assert closest_approach(segments, track.last) >= TRAIL_GAP_PX - 1e-6


def test_trail_keeps_its_distance_from_an_elongated_target() -> None:
    track = Track(1, [at(0, 60, 100), at(1, 90, 100), at(2, 120, 100, width=24, height=4)])

    segments = trail_segments(track, track.last, TRAIL_FRAMES)

    assert closest_approach(segments, track.last) == pytest.approx(TRAIL_GAP_PX, abs=0.5)


def test_trail_covers_only_the_trail_duration() -> None:
    track = Track(1, line(0, 40, (10, 100), (5, 0)))

    segments = trail_segments(track, track.last, 10)

    assert min(a[0] for a, _ in segments) == track.points[-11].x


def test_single_position_track_has_no_trail() -> None:
    track = Track(1, [at(3, 100, 100)])

    assert trail_segments(track, track.last, TRAIL_FRAMES) == []


def test_zoom_window_exists_only_for_selected_tracks_and_is_named_after_its_clip() -> None:
    faint = Track(1, [Detection(f, 100 + 5 * f, 100, 99 + 5 * f, 99, 3, 3, 9, 20.0) for f in range(8)])
    bright = Track(2, [Detection(f, 100 + 5 * f, 150, 99 + 5 * f, 149, 3, 3, 400, 120.0) for f in range(8)])
    clips = [ClipWindow(0, 50, Path("split/01_0m00s00.mp4")), ClipWindow(0, 50, Path("split/02_0m00s00.mp4"))]

    windows = zoom_windows([faint, bright], clips, INFO, RenderConfig())

    assert [(w.first, w.last, w.path, w.slowdown) for w in windows] == [
        (0, 50, Path("split/01_0m00s00_zoom.mp4"), SLOW_MOTION)
    ]
    assert windows[0].view is not None


def test_zoom_none_adds_no_window() -> None:
    track = Track(1, line(0, 8, (100, 100), (5, 0)))

    assert zoom_windows([track], [ClipWindow(0, 20, Path("a.mp4"))], INFO, RenderConfig(zoom="none")) == []
