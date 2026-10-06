from __future__ import annotations

from batdetect.spans import Span, covered_frames, without_spans
from helpers import contiguous, detection


def test_ignored_frames_keep_their_key_with_no_detection() -> None:
    detections = contiguous([detection(f, 10, 10) for f in range(5) for _ in range(f + 1)], 5)

    kept = without_spans(detections, [Span(1, 2)])

    assert {f: len(found) for f, found in kept.items()} == {0: 1, 1: 0, 2: 0, 3: 4, 4: 5}


def test_a_span_covers_both_of_its_ends() -> None:
    assert covered_frames([Span(3, 5), Span(9, 9)]) == {3, 4, 5, 9}


def test_many_short_spans_empty_exactly_their_frames() -> None:
    spans = [Span(f, f + 1) for f in range(0, 10_000, 4)]

    kept = without_spans(contiguous([detection(f, 10, 10) for f in range(10_000)], 10_000), spans)

    assert [f for f, found in kept.items() if found][:4] == [2, 3, 6, 7]
    assert sum(1 for found in kept.values() if not found) == 5_000
