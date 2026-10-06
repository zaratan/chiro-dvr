from __future__ import annotations

import itertools
import json

import pytest

from batdetect.damage import Probe, check_frame_count, damaged_spans, parse_probe
from batdetect.spans import Span
from batdetect.video import VideoError

STEP = 333


def probe_text(count: int, keys: set[int], logs: dict[int, list[str]], missing_after: int | None = None) -> str:
    frames: list[dict[str, object]] = []
    for k in range(count):
        pts = k * STEP + (13 * STEP if missing_after is not None and k > missing_after else 0)
        entry: dict[str, object] = {"key_frame": int(k in keys), "pts": pts}
        if k in logs:
            entry["logs"] = [{"message": m} for m in logs[k]]
        frames.append(entry)
    return json.dumps({"frames": frames})


def probe(count: int, keys: set[int], errors: set[int]) -> Probe:
    return parse_probe(probe_text(count, keys, {e: ["error while decoding MB 1 2, bytestream -6"] for e in errors}))


def test_any_decoder_error_message_marks_the_frame() -> None:
    text = probe_text(10, {0}, {4: ["left block unavailable for requested intra4x4 mode -1"]})

    assert parse_probe(text).error_frames == (4,)


def test_frame_after_a_jump_in_timestamps_is_damaged() -> None:
    result = parse_probe(probe_text(40, {0, 30}, {}, missing_after=11))

    assert result.gap_frames == (12,)
    assert damaged_spans(result, 0) == [Span(12, 29)]


def test_damage_lasts_until_the_frame_before_the_next_key_frame() -> None:
    assert damaged_spans(probe(45, {0, 15, 30}, {17}), 0) == [Span(17, 29)]


def test_damage_in_the_last_group_lasts_until_the_end() -> None:
    assert damaged_spans(probe(45, {0, 15, 30}, {33}), 0) == [Span(33, 44)]


def test_two_errors_in_one_group_make_one_span() -> None:
    assert damaged_spans(probe(45, {0, 15, 30}, {17, 22}), 0) == [Span(17, 29)]


def test_error_on_the_key_frame_after_a_span_extends_it() -> None:
    assert damaged_spans(probe(45, {0, 15, 30}, {17, 30}), 0) == [Span(17, 44)]


def test_error_on_the_last_frame_gives_a_one_frame_span() -> None:
    assert damaged_spans(probe(45, {0, 15, 30}, {44}), 0) == [Span(44, 44)]


def test_video_without_frames_has_no_damage() -> None:
    assert damaged_spans(parse_probe("{}"), 0) == []


def test_frame_counts_that_differ_fail_the_video() -> None:
    check_frame_count(8998, 8998)

    with pytest.raises(VideoError, match="8998 frames but OpenCV read 8997"):
        check_frame_count(8998, 8997)


def test_unknown_timestamp_between_two_frames_is_not_a_jump() -> None:
    text = json.dumps({"frames": [{"key_frame": 0, "pts": p} for p in (0, 1000, None, 3000, 4000)]})

    assert parse_probe(text).gap_frames == ()


def test_repeated_timestamps_do_not_mark_every_other_frame() -> None:
    text = json.dumps({"frames": [{"key_frame": 0, "pts": p} for p in (0, 0, 1000, 1000, 2000, 2000)]})

    assert parse_probe(text).gap_frames == ()


@pytest.mark.parametrize("text", ["not json", "[]", '{"frames": 3}', '{"frames": [1, 2]}'])
def test_unreadable_probe_output_fails_the_video(text: str) -> None:
    with pytest.raises(VideoError, match="ffprobe output"):
        parse_probe(text)


@pytest.mark.parametrize("margin", [0, 4, 15])
@pytest.mark.parametrize("errors", [(0,), (14, 15), (17, 30, 44), (3, 9, 16, 22, 31, 38, 44), tuple(range(45))])
def test_damaged_spans_never_overlap_and_stay_inside_the_video(errors: tuple[int, ...], margin: int) -> None:
    spans = damaged_spans(probe(45, {0, 15, 30}, set(errors)), margin)

    assert all(a.last + 1 < b.first for a, b in itertools.pairwise(spans))
    assert all(0 <= s.first <= s.last <= 44 for s in spans)


def test_margin_widens_both_sides_before_spans_merge() -> None:
    assert damaged_spans(probe(60, {0, 15, 30, 45}, {17, 35}), 3) == [Span(14, 47)]
    assert damaged_spans(probe(60, {0, 15, 30, 45}, {17, 37}), 3) == [Span(14, 32), Span(34, 47)]


def test_margin_stays_inside_the_video() -> None:
    assert damaged_spans(probe(45, {0, 15, 30}, {2, 40}), 15) == [Span(0, 44)]
    assert damaged_spans(probe(90, {0, 15, 30, 45, 60, 75}, {2}), 15) == [Span(0, 29)]
