from __future__ import annotations

import pytest

from batdetect.damage import Probe
from batdetect.detect import Detection
from batdetect.exclusion import exclude
from batdetect.spans import Span
from batdetect.stability import StabilityConfig
from batdetect.video import VideoError
from helpers import contiguous, detection

FPS = 10.0
FRAMES = 60
CFG = StabilityConfig(max_blobs=20, pad_s=0.2)
KEYS = (0, 15, 30, 45)


def crowd(counts: dict[int, int], usual: int = 1) -> dict[int, list[Detection]]:
    return contiguous([detection(f, 10 * i, 10) for f in range(FRAMES) for i in range(counts.get(f, usual))], FRAMES)


def probe(errors: tuple[int, ...]) -> Probe:
    return Probe(FRAMES, errors, (), KEYS)


def test_damaged_frames_cannot_make_a_span_unstable() -> None:
    analysis = exclude(crowd({17: 200, 18: 200}), probe((17,)), 0, FPS, CFG)

    assert analysis.damaged == [Span(17, 29)]
    assert analysis.unstable == []


def test_damaged_frames_do_not_lower_the_usual_blob_count() -> None:
    counts = dict.fromkeys(range(45), 0) | {50: 79}

    analysis = exclude(crowd(counts, usual=10), probe((0, 15, 30)), 0, FPS, CFG)

    assert analysis.unstable == []


def test_detections_inside_damaged_spans_are_dropped_but_frames_remain() -> None:
    analysis = exclude(crowd({}), probe((33,)), 0, FPS, CFG)

    assert sorted(analysis.detections) == list(range(FRAMES))
    assert [f for f, found in analysis.detections.items() if not found] == list(range(33, 45))


def test_real_camera_movement_outside_damage_is_still_unstable() -> None:
    analysis = exclude(crowd({17: 200, 50: 200}), probe((17,)), 0, FPS, CFG)

    assert analysis.unstable == [Span(48, 52)]
    assert analysis.ignored_spans == [Span(17, 29), Span(48, 52)]


def test_detections_read_by_opencv_must_match_the_probed_frames() -> None:
    with pytest.raises(VideoError, match="frame numbers do not match"):
        exclude(crowd({}), Probe(FRAMES + 1, (), (), KEYS), 0, FPS, CFG)


def test_time_both_damaged_and_unstable_is_not_counted_twice() -> None:
    analysis = exclude(crowd({33: 200}), probe((17,)), 0, FPS, CFG)

    assert analysis.ignored_spans == [Span(17, 29), Span(31, 35)]
    assert len(analysis.ignored_frames) == 13 + 5

    overlapping = exclude(crowd({31: 200}), probe((17,)), 0, FPS, CFG)

    assert overlapping.ignored_spans == [Span(17, 29), Span(29, 33)]
    assert len(overlapping.ignored_frames) == len(range(17, 34))


def test_frames_after_a_timestamp_jump_count_as_reported_damage() -> None:
    analysis = exclude(crowd({}), Probe(FRAMES, (17,), (40,), KEYS), 0, FPS, CFG)

    assert analysis.reported_frames == 2
    assert analysis.damaged == [Span(17, 29), Span(40, 44)]


def test_crowded_frame_whose_background_saw_the_damage_cannot_make_a_span_unstable() -> None:
    analysis = exclude(crowd({14: 200, 32: 200}), probe((17,)), 3, FPS, CFG)

    assert analysis.damaged == [Span(14, 32)]
    assert analysis.unstable == []
