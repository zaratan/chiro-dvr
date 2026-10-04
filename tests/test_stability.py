from __future__ import annotations

from collections.abc import Callable

import pytest

from batdetect.detect import Detection
from batdetect.stability import Span, StabilityConfig, unstable_spans, without_spans
from helpers import detection

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


def test_ignored_frames_keep_their_key_with_no_detection() -> None:
    detections = crowd([1, 2, 3, 4, 5])

    kept = without_spans(detections, [Span(1, 2)])

    assert {f: len(found) for f, found in kept.items()} == {0: 1, 1: 0, 2: 0, 3: 4, 4: 5}


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
