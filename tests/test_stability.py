from __future__ import annotations

from collections.abc import Callable

import numpy as np
import pytest

from batdetect.detect import DetectConfig, Detection, detect_frames
from batdetect.spans import Span, without_spans
from batdetect.stability import StabilityConfig, unstable_spans
from batdetect.track import TrackConfig, track_detections
from batdetect.video import GrayFrame
from helpers import background, detection

FPS = 10.0
CFG = StabilityConfig(max_blobs=20, pad_s=0.2)


def crowd(counts: list[int]) -> dict[int, list[Detection]]:
    return {f: [detection(f, 10 * i, 10) for i in range(n)] for f, n in enumerate(counts)}


def test_frames_below_the_limit_are_kept() -> None:
    assert unstable_spans(crowd([1] * 20 + [25] + [1] * 20), FPS, CFG) == []


def test_a_crowded_frame_is_ignored_with_its_padding() -> None:
    assert unstable_spans(crowd([0] * 20 + [20] + [0] * 20), FPS, CFG) == [Span(18, 22)]


def test_padding_is_rounded_to_frames_at_a_fractional_rate() -> None:
    cfg = StabilityConfig(max_blobs=20, pad_s=1.0)

    assert unstable_spans(crowd([0] * 100 + [30] + [0] * 100), 30.03, cfg) == [Span(70, 130)]


def test_the_limit_rises_with_the_usual_number_of_blobs() -> None:
    noisy = [10] * 40 + [70] + [10] * 40

    assert unstable_spans(crowd(noisy), FPS, CFG) == []
    assert unstable_spans(crowd([*noisy[:40], 80, *noisy[41:]]), FPS, CFG) == [Span(38, 42)]


def test_close_crowded_frames_form_one_span() -> None:
    counts = [0] * 40
    counts[10] = counts[15] = 30

    assert unstable_spans(crowd(counts), FPS, CFG) == [Span(8, 17)]


def test_spans_separated_by_exactly_the_padding_are_merged() -> None:
    counts = [0] * 40
    counts[10] = counts[17] = 30

    assert unstable_spans(crowd(counts), FPS, CFG) == [Span(8, 19)]


def test_spans_separated_by_one_frame_more_than_the_padding_stay_apart() -> None:
    counts = [0] * 40
    counts[10] = counts[18] = 30

    assert unstable_spans(crowd(counts), FPS, CFG) == [Span(8, 12), Span(16, 20)]


def test_spans_stop_at_the_first_and_last_frames() -> None:
    counts = [30] + [0] * 20 + [30]

    assert unstable_spans(crowd(counts), FPS, CFG) == [Span(0, 2), Span(19, 21)]


def test_zero_max_blobs_disables_the_filter() -> None:
    assert unstable_spans(crowd([0] * 20 + [500] + [0] * 20), FPS, StabilityConfig(max_blobs=0)) == []


@pytest.mark.parametrize(
    "build",
    [
        lambda: StabilityConfig(max_blobs=-1),
        lambda: StabilityConfig(pad_s=-0.1),
        lambda: StabilityConfig(pad_s=float("nan")),
        lambda: StabilityConfig(pad_s=float("inf")),
    ],
)
def test_invalid_stability_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match="must be"):
        build()


JUMP_FRAME = 45


def frames_with_brightness_jump(jump: int) -> list[GrayFrame]:
    frames = [background(160, 120, seed=i) for i in range(2 * JUMP_FRAME)]
    return [
        f if i < JUMP_FRAME else np.clip(f.astype(np.int16) + jump, 0, 255).astype(np.uint8)
        for i, f in enumerate(frames)
    ]


@pytest.mark.parametrize("jump", [20, 30, 40])
def test_global_brightness_jump_leaves_no_track_with_the_default_settings(jump: int) -> None:
    detections = detect_frames(frames_with_brightness_jump(jump), 30.0, DetectConfig(), 3.0)
    spans = unstable_spans(detections, 30.0, StabilityConfig())

    assert track_detections(without_spans(detections, spans), TrackConfig()) == []


def test_large_brightness_jump_is_ignored_around_the_jump_rather_than_tracked() -> None:
    detections = detect_frames(frames_with_brightness_jump(30), 30.0, DetectConfig(), 3.0)

    spans = unstable_spans(detections, 30.0, StabilityConfig())

    assert spans
    assert all(s.first <= JUMP_FRAME + 15 and s.last >= JUMP_FRAME - 15 for s in spans)
    assert sum(s.last - s.first + 1 for s in spans) <= 3 * 30


def test_moderate_brightness_jump_gives_a_few_blobs_but_ignores_nothing() -> None:
    detections = detect_frames(frames_with_brightness_jump(20), 30.0, DetectConfig(), 3.0)

    assert any(detections.values())
    assert unstable_spans(detections, 30.0, StabilityConfig()) == []
