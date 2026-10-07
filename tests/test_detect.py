from __future__ import annotations

from collections.abc import Callable, Sequence
from dataclasses import replace

import cv2
import numpy as np
import numpy.typing as npt
import pytest

import batdetect.detect
from batdetect.detect import (
    MIN_NOISE_SAMPLES,
    DetectConfig,
    Detection,
    Region,
    detect_frames,
    find_blobs,
    osd_mask,
    raised_threshold,
    target_blur,
)
from batdetect.median import temporal_median
from batdetect.noise import temporal_noise
from batdetect.video import GrayFrame
from helpers import background, moving_square_frames, with_square

FPS = 30.0
SCALE = 3.0
UNFILTERED = DetectConfig(threshold=25, noise_factor=0, target_sigma=0)


def test_dark_moving_spot_is_detected_on_every_frame_at_its_centroid() -> None:
    frames = moving_square_frames(40, start=(20, 60), step=(3, 0))

    detections = detect_frames(frames, FPS, UNFILTERED, SCALE)

    assert sorted(detections) == list(range(40))
    for frame, found in detections.items():
        assert len(found) == 1
        assert found[0].x == pytest.approx(SCALE * (21 + 3 * frame), abs=1.5)
        assert found[0].y == pytest.approx(SCALE * 61, abs=1.5)


def test_bright_moving_spot_is_detected_too() -> None:
    frames = [with_square(background(160, 120, seed=i), 20 + 3 * i, 60, delta=50) for i in range(40)]

    detections = detect_frames(frames, FPS, DetectConfig(), SCALE)

    assert all(len(found) == 1 for found in detections.values())


def test_global_brightness_jump_is_compensated_at_a_fixed_threshold() -> None:
    frames = [background(160, 120, seed=i) for i in range(40)]
    frames = [f if i < 20 else np.clip(f.astype(np.int16) + 30, 0, 255).astype(np.uint8) for i, f in enumerate(frames)]

    detections = detect_frames(frames, FPS, UNFILTERED, SCALE)

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

    assert all(found == [] for found in detect_frames(frames, FPS, replace(UNFILTERED, min_area=90), SCALE).values())
    assert all(found == [] for found in detect_frames(frames, FPS, replace(UNFILTERED, max_area=72), SCALE).values())


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

    merged = detect_frames(frames, FPS, replace(UNFILTERED, merge_radius=6), SCALE)
    split = detect_frames(frames, FPS, replace(UNFILTERED, merge_radius=0), SCALE)

    assert all(len(found) == 1 for found in merged.values())
    assert all(found[0].area == 18 * SCALE**2 for found in merged.values())
    assert all(len(found) == 2 for found in split.values())


def test_background_window_too_short_for_the_frame_rate_is_rejected() -> None:
    with pytest.raises(ValueError, match="fewer than"):
        DetectConfig(bg_window_s=0.01).half_window(FPS)


def test_background_step_leaving_under_three_frames_is_rejected_rather_than_using_one_frame_as_background() -> None:
    with pytest.raises(ValueError, match="bg_step=40 keeps fewer than"):
        DetectConfig(bg_step=40).half_window(FPS)


def test_background_step_leaving_exactly_three_frames_is_accepted_and_one_more_is_rejected() -> None:
    assert DetectConfig(bg_step=15).half_window(FPS) == 15
    with pytest.raises(ValueError, match="bg_step=16"):
        DetectConfig(bg_step=16).half_window(FPS)


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
        lambda: DetectConfig(noise_factor=-1),
        lambda: DetectConfig(noise_factor=float("nan")),
        lambda: DetectConfig(target_sigma=-1),
        lambda: DetectConfig(target_sigma=float("inf")),
    ],
)
def test_invalid_detect_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match=r"must|expected"):
        build()


def blobs_one_by_one(residual: npt.NDArray[np.float64], cfg: DetectConfig, scale: float) -> list[Detection]:
    above = np.abs(residual) > cfg.threshold
    size = 2 * round(cfg.merge_radius / scale) + 1
    kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (size, size))
    merged = np.asarray(cv2.morphologyEx(above.astype(np.uint8), cv2.MORPH_CLOSE, kernel), dtype=np.uint8)
    count, labels_mat, stats_mat, centroids_mat = cv2.connectedComponentsWithStats(merged)
    labels = np.asarray(labels_mat)
    stats = np.asarray(stats_mat, dtype=np.int64)
    centroids = np.asarray(centroids_mat, dtype=np.float64)
    found: list[Detection] = []
    for i in range(1, count):
        pixels = above & (labels == i)
        area = float(pixels.sum()) * scale**2
        if not cfg.min_area <= area <= cfg.max_area:
            continue
        left, top, width, height = (float(v) * scale for v in stats[i, :4])
        cx, cy = ((float(v) + 0.5) * scale - 0.5 for v in centroids[i])
        amplitude = float(np.abs(residual[pixels]).max())
        found.append(Detection(7, cx, cy, left, top, width, height, area, amplitude))
    return found


def many_blobs_with_bridged_pairs() -> npt.NDArray[np.float64]:
    rng = np.random.default_rng(3)
    residual = rng.uniform(-10, 10, size=(160, 320))
    for row in range(10):
        for col in range(20):
            y, x = 16 * row + (0 if row == 0 else 3), 16 * col + 3
            size = 3 if col % 5 == 0 else int(rng.integers(1, 4))
            sign = 1 if rng.random() < 0.5 else -1
            residual[y : y + size, x : x + size] = sign * rng.uniform(30, 90, size=(size, size))
            if col % 5 == 0:
                residual[y : y + size, x + size] = 10
                residual[y : y + size, x + size + 1 : x + 2 * size + 1] = -rng.uniform(30, 90, size=(size, size))
    return residual


def test_many_blobs_are_measured_as_one_by_one() -> None:
    residual = many_blobs_with_bridged_pairs()

    found = find_blobs(residual, 7, DetectConfig(), SCALE)

    assert len(found) == 200
    assert found == blobs_one_by_one(residual, DetectConfig(), SCALE)


FLICKER_X = (100, 140)
LEAF_SPACING = 6
LEAVES = (20, 7)


def frames_with_flickering_strip(count: int) -> list[GrayFrame]:
    rng = np.random.default_rng(11)
    frames: list[GrayFrame] = []
    for i in range(count):
        frame = (200 + rng.integers(-1, 2, (120, 160))).astype(np.float64)
        frame[::LEAF_SPACING, FLICKER_X[0] : FLICKER_X[1] : LEAF_SPACING] += rng.normal(0, 15, LEAVES)
        frames.append(with_square(np.clip(frame, 0, 255).astype(np.uint8), 5 + 2 * i, 30, delta=-20))
    return frames


def in_strip(found: dict[int, list[Detection]]) -> int:
    return sum(FLICKER_X[0] - 3 <= d.x / SCALE <= FLICKER_X[1] + 3 for dets in found.values() for d in dets)


def test_flickering_pixels_raise_their_own_threshold_while_calm_ones_keep_the_floor() -> None:
    frames = frames_with_flickering_strip(40)

    fixed = detect_frames(frames, FPS, DetectConfig(threshold=12, noise_factor=0, target_sigma=0), SCALE)
    per_pixel = detect_frames(frames, FPS, DetectConfig(threshold=12, noise_factor=4, target_sigma=0), SCALE)

    outside = [[d for d in dets if d.x / SCALE < FLICKER_X[0] - 3] for dets in per_pixel.values()]
    assert in_strip(per_pixel) < in_strip(fixed) / 4
    assert all(len(found) == 1 and abs(found[0].y / SCALE - 31) < 2 for found in outside)


def test_zero_noise_factor_never_computes_the_noise_map(monkeypatch: pytest.MonkeyPatch) -> None:
    def refuse(*_: object) -> None:
        raise AssertionError("noise map computed")

    monkeypatch.setattr(batdetect.detect, "temporal_noise", refuse)

    detections = detect_frames(moving_square_frames(40), FPS, DetectConfig(noise_factor=0), SCALE)

    assert all(len(found) == 1 for found in detections.values())


def test_truncated_window_at_the_video_edges_falls_back_to_the_fixed_threshold(monkeypatch: pytest.MonkeyPatch) -> None:
    sample_counts: list[int] = []
    measure = batdetect.detect.temporal_noise

    def spy(
        samples: Sequence[GrayFrame], background: npt.NDArray[np.float64], gains: Sequence[float]
    ) -> npt.NDArray[np.float64]:
        sample_counts.append(len(samples))
        return measure(samples, background, gains)

    monkeypatch.setattr(batdetect.detect, "temporal_noise", spy)

    detections = detect_frames(moving_square_frames(40), FPS, DetectConfig(noise_factor=4), SCALE)

    assert len(detections) == 40
    assert min(sample_counts) >= MIN_NOISE_SAMPLES
    assert 0 < len(sample_counts) < 40


def test_raising_only_the_pixels_above_the_floor_keeps_the_same_pixels_as_raising_all_of_them() -> None:
    rng = np.random.default_rng(5)
    samples = [np.clip(rng.normal(120, 6, (60, 80)), 0, 255).astype(np.uint8) for _ in range(11)]
    background = temporal_median(samples)
    levels = [float(f.mean()) for f in samples]
    gains = [level - float(np.mean(levels)) for level in levels]
    residual = samples[5] - background - gains[5]
    cfg = DetectConfig(threshold=8, noise_factor=2)

    everywhere = np.maximum(cfg.threshold, cfg.noise_factor * temporal_noise(samples, background, gains))
    raised = raised_threshold(residual, samples, background, gains, cfg)

    assert ((np.abs(residual) > everywhere) != (np.abs(residual) > cfg.threshold)).any()
    assert np.array_equal(np.abs(residual) > raised, np.abs(residual) > everywhere)


def test_a_residual_below_the_floor_everywhere_keeps_the_plain_floor() -> None:
    samples = [np.full((20, 30), 120, dtype=np.uint8) for _ in range(11)]
    residual = np.full((20, 30), 3.0)

    cfg = DetectConfig(noise_factor=8)

    assert raised_threshold(residual, samples, temporal_median(samples), [0.0] * 11, cfg) == cfg.threshold


def frames_with_salt(count: int) -> list[GrayFrame]:
    rng = np.random.default_rng(13)
    frames: list[GrayFrame] = []
    for i in range(count):
        frame = np.full((120, 160), 120, dtype=np.int16)
        ys, xs = rng.integers(0, 120, 30), rng.integers(60, 160, 30)
        frame[ys, xs] += rng.choice(np.array([-40, 40]), 30)
        frames.append(with_square(frame.astype(np.uint8), 5 + i, 30, delta=-30))
    return frames


def test_single_pixel_noise_is_smoothed_away_while_a_target_sized_spot_survives() -> None:
    frames = frames_with_salt(40)

    sharp = detect_frames(frames, FPS, DetectConfig(threshold=12, noise_factor=0, target_sigma=0), SCALE)
    smoothed = detect_frames(frames, FPS, DetectConfig(threshold=12, noise_factor=0, target_sigma=SCALE), SCALE)

    assert sum(len(found) for found in sharp.values()) > 40 * 10
    assert all(len(found) == 1 and abs(found[0].y / SCALE - 31) < 2 for found in smoothed.values())


def test_peak_amplitude_is_measured_on_the_filtered_residual() -> None:
    frames = moving_square_frames(40)

    sharp = detect_frames(frames, FPS, UNFILTERED, SCALE)
    smoothed = detect_frames(frames, FPS, replace(UNFILTERED, target_sigma=SCALE), SCALE)

    assert max(d.amplitude for found in smoothed.values() for d in found) < 0.8 * min(
        d.amplitude for found in sharp.values() for d in found
    )


def test_zero_target_sigma_leaves_the_frame_as_it_is() -> None:
    frame = background(160, 120)

    assert target_blur(frame, 0) is frame


def test_blurred_osd_digits_do_not_leak_below_the_masked_band() -> None:
    rng = np.random.default_rng(17)
    frames = [background(160, 120, seed=i) for i in range(40)]
    for frame in frames:
        frame[0:8, :] = rng.choice(np.array([0, 255], dtype=np.uint8), (8, 160))
    cfg = DetectConfig(threshold=12, noise_factor=0, target_sigma=2 * SCALE, osd_regions=(Region(0, 0, 1, 8 / 120),))

    assert all(found == [] for found in detect_frames(frames, FPS, cfg, SCALE).values())


def test_default_settings_place_a_moving_spot_within_two_source_pixels() -> None:
    frames = moving_square_frames(40, start=(20, 60), step=(3, 0))

    detections = detect_frames(frames, FPS, DetectConfig(), SCALE)

    assert sorted(detections) == list(range(40))
    for frame, found in detections.items():
        assert len(found) == 1
        assert found[0].x == pytest.approx(SCALE * (21 + 3 * frame), abs=2)
        assert found[0].y == pytest.approx(SCALE * 61, abs=2)
