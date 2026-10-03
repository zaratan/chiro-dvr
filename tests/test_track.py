from __future__ import annotations

from collections.abc import Callable

import pytest

from batdetect.pipeline import Track, TrackConfig, combine, merge_twins, track_detections
from helpers import by_frame, detection, line


def test_straight_flight_gives_one_track() -> None:
    tracks = track_detections(by_frame(line(0, 10, (10, 50), (5, 0))), TrackConfig())

    assert len(tracks) == 1
    assert len(tracks[0].points) == 10


def test_two_crossing_flights_stay_two_tracks() -> None:
    a = line(0, 12, (10, 10), (6, 6))
    b = line(0, 12, (10, 76), (6, -6))

    tracks = track_detections(by_frame(a, b), TrackConfig())

    assert len(tracks) == 2
    assert {round(t.first.y) for t in tracks} == {10, 76}
    assert {round(t.last.y) for t in tracks} == {76, 10}


def test_gap_shorter_than_max_gap_is_bridged() -> None:
    flight = [d for d in line(0, 14, (10, 50), (4, 0)) if d.frame not in {6, 7, 8}]

    tracks = track_detections(by_frame(flight), TrackConfig(max_gap=6))

    assert len(tracks) == 1


def test_gap_longer_than_max_gap_splits_the_track() -> None:
    flight = [d for d in line(0, 20, (10, 50), (4, 0)) if not 6 <= d.frame <= 13]

    tracks = track_detections(by_frame(flight), TrackConfig(max_gap=6, max_jump=10))

    assert len(tracks) == 2


def test_tracks_below_min_hits_are_dropped() -> None:
    tracks = track_detections(by_frame(line(0, 4, (10, 50), (5, 0))), TrackConfig(min_hits=5))

    assert tracks == []


def test_flickering_spot_that_does_not_travel_is_dropped() -> None:
    tracks = track_detections(by_frame(line(0, 20, (50, 50), (0.2, 0))), TrackConfig(min_travel=15))

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

    tracks = track_detections(by_frame(body, wing), TrackConfig(twin_distance=12))

    assert len(tracks) == 1
    assert len(tracks[0].points) == 10


def test_two_animals_flying_apart_are_not_merged() -> None:
    a = Track(1, line(0, 10, (10, 50), (6, 0)))
    b = Track(2, line(0, 10, (14, 54), (6, 3)))

    assert len(merge_twins([a, b], max_distance=12)) == 2


def test_tracks_that_never_coexist_are_not_twins() -> None:
    a = Track(1, line(0, 5, (10, 50), (6, 0)))
    b = Track(2, line(10, 5, (12, 50), (6, 0)))

    assert len(merge_twins([a, b], max_distance=12)) == 2


def test_combined_fragment_is_area_weighted_and_boxes_both() -> None:
    big = detection(0, 10, 10, area=30)
    small = detection(0, 20, 10, area=10)

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
    ],
)
def test_invalid_track_config_is_rejected(build: Callable[[], object]) -> None:
    with pytest.raises(ValueError, match="must be"):
        build()
