from __future__ import annotations

from collections.abc import Callable

import numpy as np
import pytest

from batdetect.detect import DetectConfig, Region, detect_frames, osd_mask
from helpers import background, moving_square_frames, with_square

FPS = 30.0
SCALE = 3.0


def test_dark_moving_spot_is_detected_on_every_frame_at_its_centroid() -> None:
    frames = moving_square_frames(40, start=(20, 60), step=(3, 0))

    detections = detect_frames(frames, FPS, DetectConfig(), SCALE)

    assert sorted(detections) == list(range(40))
    for frame, found in detections.items():
        assert len(found) == 1
        assert found[0].x == pytest.approx(SCALE * (21 + 3 * frame), abs=1.5)
        assert found[0].y == pytest.approx(SCALE * 61, abs=1.5)


def test_bright_moving_spot_is_detected_too() -> None:
    frames = [with_square(background(160, 120, seed=i), 20 + 3 * i, 60, delta=50) for i in range(40)]

    detections = detect_frames(frames, FPS, DetectConfig(), SCALE)

    assert all(len(found) == 1 for found in detections.values())


def test_global_brightness_jump_is_not_motion() -> None:
    frames = [background(160, 120, seed=i) for i in range(40)]
    frames = [f if i < 20 else np.clip(f.astype(np.int16) + 30, 0, 255).astype(np.uint8) for i, f in enumerate(frames)]

    detections = detect_frames(frames, FPS, DetectConfig(), SCALE)

    assert all(found == [] for found in detections.values())


def test_spot_inside_the_osd_band_is_ignored() -> None:
    frames = moving_square_frames(40, start=(20, 2), step=(3, 0))

    detections = detect_frames(frames, FPS, DetectConfig(osd_regions=(Region(0, 0, 1, 0.07),)), SCALE)

    assert all(found == [] for found in detections.values())


def test_osd_region_mask_is_rounded_outwards() -> None:
    mask = osd_mask(10, 10, DetectConfig(osd_regions=(Region(0.15, 0.15, 0.35, 0.35),)))

    assert not mask[1:4, 1:4].any()
    assert mask[4, 4]
    assert mask[0, 0]


def test_masks_covering_the_whole_frame_are_refused() -> None:
    halves = DetectConfig(osd_regions=(Region(0, 0, 1, 0.5), Region(0, 0.5, 1, 1)))

    with pytest.raises(ValueError, match="whole frame"):
        detect_frames(moving_square_frames(5), FPS, halves, SCALE)


def test_no_mask_by_default() -> None:
    assert osd_mask(10, 10, DetectConfig()).all()


def test_blobs_outside_area_bounds_are_ignored() -> None:
    frames = moving_square_frames(40)

    assert all(found == [] for found in detect_frames(frames, FPS, DetectConfig(min_area=90), SCALE).values())
    assert all(found == [] for found in detect_frames(frames, FPS, DetectConfig(max_area=72), SCALE).values())


def test_video_shorter_than_the_background_window_is_still_analysed() -> None:
    frames = moving_square_frames(12, step=(4, 0))

    detections = detect_frames(frames, FPS, DetectConfig(bg_window_s=1.0), SCALE)

    assert sorted(detections) == list(range(12))
    assert all(len(found) == 1 for found in detections.values())


def test_no_frames_gives_no_detections() -> None:
    assert detect_frames([], FPS, DetectConfig(), SCALE) == {}


def test_fragments_closer_than_merge_radius_become_one_blob() -> None:
    frames = [
        with_square(with_square(background(160, 120, seed=i), 20 + 3 * i, 60), 20 + 3 * i + 5, 60) for i in range(40)
    ]

    merged = detect_frames(frames, FPS, DetectConfig(merge_radius=6), SCALE)
    split = detect_frames(frames, FPS, DetectConfig(merge_radius=0), SCALE)

    assert all(len(found) == 1 for found in merged.values())
    assert all(found[0].area == 18 * SCALE**2 for found in merged.values())
    assert all(len(found) == 2 for found in split.values())


def test_background_window_too_short_for_the_frame_rate_is_rejected() -> None:
    with pytest.raises(ValueError, match="fewer than"):
        DetectConfig(bg_window_s=0.01).half_window(FPS)


@pytest.mark.parametrize(
    "build",
    [
        lambda: DetectConfig(threshold=0),
        lambda: DetectConfig(min_area=0),
        lambda: DetectConfig(min_area=10, max_area=5),
        lambda: DetectConfig(bg_window_s=0),
        lambda: DetectConfig(bg_step=0),
        lambda: DetectConfig(merge_radius=-1),
        lambda: DetectConfig(work_width=8),
    ],
)
def test_invalid_detect_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match=r"must|expected"):
        build()
