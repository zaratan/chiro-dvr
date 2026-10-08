from __future__ import annotations

import math
from collections.abc import Callable

import pytest

import helpers
from batdetect.detect import Detection
from batdetect.track import Track, TrackConfig, combine, merge_twins, track_detections
from helpers import by_frame, contiguous

S = 3


def line(start_frame: int, frames: int, start: tuple[float, float], step: tuple[float, float]) -> list[Detection]:
    return helpers.line(start_frame, frames, (start[0] * S, start[1] * S), (step[0] * S, step[1] * S))


def detection(frame: int, x: float, y: float) -> Detection:
    return helpers.detection(frame, x * S, y * S)


def test_straight_flight_gives_one_track() -> None:
    tracks = track_detections(by_frame(line(0, 10, (10, 50), (5, 0))), TrackConfig())

    assert len(tracks) == 1
    assert len(tracks[0].points) == 10


def test_two_crossing_flights_stay_two_tracks() -> None:
    a = line(0, 12, (10, 10), (6, 6))
    b = line(0, 12, (10, 76), (6, -6))

    tracks = track_detections(by_frame(a, b), TrackConfig())

    assert len(tracks) == 2
    assert {round(t.first.y) for t in tracks} == {10 * S, 76 * S}
    assert {round(t.last.y) for t in tracks} == {76 * S, 10 * S}


def test_gap_shorter_than_max_gap_is_bridged() -> None:
    flight = [d for d in line(0, 14, (10, 50), (4, 0)) if d.frame not in {6, 7, 8}]

    tracks = track_detections(by_frame(flight), TrackConfig(max_gap=6))

    assert len(tracks) == 1


def test_gap_longer_than_max_gap_splits_the_track() -> None:
    flight = [d for d in line(0, 20, (10, 50), (4, 0)) if not 6 <= d.frame <= 13]

    tracks = track_detections(by_frame(flight), TrackConfig(max_gap=6, max_jump=10 * S))

    assert len(tracks) == 2


@pytest.mark.parametrize(("missing", "expected_tracks"), [(6, 1), (7, 2)])
def test_track_survives_exactly_max_gap_empty_frames(missing: int, expected_tracks: int) -> None:
    flight = [d for d in line(0, 30, (10, 50), (4, 0)) if not 10 <= d.frame < 10 + missing]

    tracks = track_detections(contiguous(flight, 30), TrackConfig(max_gap=6, twin_distance=0))

    assert len(tracks) == expected_tracks


def test_saccade_prediction_error_stays_under_a_third_of_the_long_step() -> None:
    xs = [0, 20, 30, 40, 60, 70, 80, 100, 110, 120, 140, 150]
    points = [helpers.detection(f, x, 0) for f, x in enumerate(xs)]

    errors = [abs(Track(1, points[:k]).predicted(k)[0] - xs[k]) for k in range(3, len(xs))]

    assert max(errors) <= 20 / 3 + 1e-9


def test_prediction_falls_back_to_the_last_two_points_on_a_young_track() -> None:
    track = Track(1, [helpers.detection(0, 0, 0), helpers.detection(1, 10, 0)])

    assert track.predicted(2) == (20, 0)


def test_prediction_spans_three_frames_even_across_a_gap() -> None:
    xs = {0: 0, 1: 10, 2: 20, 5: 80, 6: 90}
    track = Track(1, [helpers.detection(f, x, 0) for f, x in xs.items()])

    assert track.predicted(7) == pytest.approx((90 + (90 - 20) / 4, 0))


def test_tracks_below_min_hits_are_dropped() -> None:
    tracks = track_detections(by_frame(line(0, 4, (10, 50), (5, 0))), TrackConfig(min_hits=5))

    assert tracks == []


def test_flickering_spot_that_does_not_travel_is_dropped() -> None:
    tracks = track_detections(by_frame(line(0, 20, (50, 50), (0.2, 0))), TrackConfig(min_travel=15 * S))

    assert tracks == []


def test_tracks_are_numbered_in_chronological_order() -> None:
    late = line(30, 8, (10, 20), (5, 0))
    early = line(0, 8, (10, 90), (5, 0))

    tracks = track_detections(by_frame(late, early), TrackConfig())

    assert [t.id for t in tracks] == [1, 2]
    assert tracks[0].first.frame == 0


def test_established_track_keeps_its_detection_over_a_closer_newborn_track() -> None:
    flight = line(0, 6, (10, 50), (8, 0)) + line(6, 4, (54, 50), (4, 0))
    fragment = [detection(5, 53, 51)]

    tracks = track_detections(by_frame(flight, fragment), TrackConfig(twin_distance=0))

    assert len(tracks) == 1
    assert len(tracks[0].points) == 10


def test_fragments_of_one_animal_are_merged_into_one_track() -> None:
    body = line(0, 10, (10, 50), (6, 0))
    wing = line(0, 10, (14, 56), (6, 0))

    tracks = track_detections(by_frame(body, wing), TrackConfig(twin_distance=12 * S))

    assert len(tracks) == 1
    assert len(tracks[0].points) == 10


def test_two_animals_flying_apart_are_not_merged() -> None:
    a = Track(1, line(0, 10, (10, 50), (6, 0)))
    b = Track(2, line(0, 10, (14, 54), (6, 3)))

    assert len(merge_twins([a, b], max_distance=12 * S)) == 2


def test_tracks_that_never_coexist_are_not_twins() -> None:
    a = Track(1, line(0, 5, (10, 50), (6, 0)))
    b = Track(2, line(10, 5, (12, 50), (6, 0)))

    assert len(merge_twins([a, b], max_distance=12 * S)) == 2


def test_combined_fragment_is_area_weighted_and_boxes_both() -> None:
    big = helpers.detection(0, 10, 10, area=30)
    small = helpers.detection(0, 20, 10, area=10)

    merged = combine(big, small)

    assert merged.x == pytest.approx(12.5)
    assert merged.area == 40
    assert (merged.left, merged.width) == (9, 13)


@pytest.mark.parametrize(
    "build",
    [
        lambda: TrackConfig(max_jump=0),
        lambda: TrackConfig(max_gap=0),
        lambda: TrackConfig(min_hits=0),
        lambda: TrackConfig(min_travel=-1),
        lambda: TrackConfig(twin_distance=-1),
        lambda: TrackConfig(max_median_turn=0),
    ],
)
def test_invalid_track_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match="must be"):
        build()


def zigzag(amplitude: float, frames: int = 12) -> list[Detection]:
    return [helpers.detection(i, 20.0 * i, 100 + (amplitude if i % 2 else -amplitude)) for i in range(frames)]


def test_gentle_zigzag_is_kept_but_a_wide_one_is_noise() -> None:
    assert len(track_detections(contiguous(zigzag(2), 12), TrackConfig())) == 1
    assert track_detections(contiguous(zigzag(15), 12), TrackConfig()) == []


def test_a_large_enough_max_median_turn_keeps_any_zigzag() -> None:
    assert len(track_detections(contiguous(zigzag(30), 12), TrackConfig(max_median_turn=math.pi))) == 1


def test_a_track_exactly_at_the_turn_limit_is_kept() -> None:
    limit = Track(1, zigzag(15)).median_turn()

    assert len(track_detections(contiguous(zigzag(15), 12), TrackConfig(max_median_turn=limit))) == 1


def test_a_curve_turning_steadily_has_a_small_median_turn() -> None:
    arc = [helpers.detection(i, 300 + 200 * math.cos(i / 10), 300 + 200 * math.sin(i / 10)) for i in range(20)]

    assert Track(1, arc).median_turn() == pytest.approx(0.1)


def test_a_spot_lit_twice_in_place_does_not_count_as_a_turn() -> None:
    points = [helpers.detection(f, 50, y) for f, y in enumerate([100, 90, 90, 80, 70, 60])]

    assert Track(1, points).median_turn() == 0


def test_tracks_too_short_to_turn_are_kept_for_lack_of_evidence() -> None:
    assert Track(1, [helpers.detection(0, 0, 0)]).median_turn() == 0
    assert Track(1, [helpers.detection(0, 0, 0), helpers.detection(1, 10, 0)]).median_turn() == 0


def test_two_sharp_turns_out_of_three_reject_a_short_track() -> None:
    points = [helpers.detection(f, x, y) for f, (x, y) in enumerate([(0, 0), (20, 0), (20, 20), (40, 20), (60, 20)])]

    assert Track(1, points).median_turn() == pytest.approx(math.pi / 2)


def test_a_gentle_zigzag_flying_west_is_still_gentle_across_the_half_turn_boundary() -> None:
    west = [helpers.detection(i, 400 - 20.0 * i, 100 + (2 if i % 2 else -2)) for i in range(12)]

    assert Track(1, west).median_turn() == pytest.approx(Track(1, zigzag(2)).median_turn())


def test_one_sharp_turn_in_a_straight_track_does_not_reject_it() -> None:
    points = [
        helpers.detection(f, x, y) for f, (x, y) in enumerate([(0, 0), (20, 0), (40, 0), (40, 20), (40, 40), (40, 60)])
    ]

    assert Track(1, points).median_turn() == 0


def test_largest_area_and_peak_amplitude_may_come_from_different_detections() -> None:
    track = Track(
        1,
        [
            Detection(0, 10, 10, 9, 9, 3, 3, 40, 22.5),
            Detection(1, 20, 10, 19, 9, 3, 3, 12, 61.0),
            Detection(2, 30, 10, 29, 9, 3, 3, 25, 30.0),
        ],
    )

    assert (track.max_area(), track.peak_amplitude()) == (40, 61.0)
