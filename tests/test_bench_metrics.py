from __future__ import annotations

import pytest

from batdetect.bench.metrics import evaluate, is_visible
from batdetect.detect import Region
from batdetect.stability import StabilityConfig
from batdetect.synthetic.injection import Observation
from batdetect.track import TrackConfig
from bench_support import INFO, MATCH, bat_along, detections_on, make_run
from helpers import detection


def test_target_under_a_detection_mask_is_not_visible() -> None:
    assert not is_visible(Observation(0, 240, 5, -40), 3, INFO, (Region(0, 0, 1, 0.1),))


def test_target_touching_the_frame_edge_is_not_visible() -> None:
    assert not is_visible(Observation(0, 2, 180, -40), 3, INFO)


def test_target_without_effective_contrast_is_not_visible() -> None:
    assert not is_visible(Observation(0, 240, 180, -3), 3, INFO)


def test_fully_detected_target_is_complete_with_no_delay() -> None:
    bat = bat_along(0, range(10, 30))

    result = evaluate(make_run([bat], detections_on(bat)), TrackConfig(), MATCH).bats[0]

    assert result.found
    assert result.completeness == 1
    assert (result.start_delay, result.end_early, result.fragments) == (0, 0, 1)


def test_late_start_and_early_end_are_counted_in_frames() -> None:
    bat = bat_along(0, range(10, 30))
    detections = detections_on(bat, skip={10, 11, 12, 28, 29})

    result = evaluate(make_run([bat], detections), TrackConfig(), MATCH).bats[0]

    assert (result.start_delay, result.end_early) == (3, 2)
    assert result.completeness == pytest.approx(15 / 20)


def test_target_on_saturated_sky_is_excluded_rather_than_missed() -> None:
    bat = bat_along(0, range(10, 30))

    evaluation = evaluate(make_run([bat], {}, effective=0), TrackConfig(), MATCH)

    assert evaluation.bats == []
    assert evaluation.not_visible == 1


def test_track_split_by_a_long_gap_counts_as_two_fragments() -> None:
    bat = bat_along(0, range(10, 40))
    detections = detections_on(bat, skip=set(range(20, 28)))

    result = evaluate(make_run([bat], detections), TrackConfig(max_gap=6), MATCH).bats[0]

    assert result.fragments == 2


def test_track_with_no_target_and_no_reference_counterpart_is_a_false_track() -> None:
    bat = bat_along(0, range(10, 30))
    stray = {f: [detection(f, 60 + 12 * (f - 50), 300)] for f in range(50, 60)}

    evaluation = evaluate(make_run([bat], detections_on(bat) | stray), TrackConfig(), MATCH)

    assert evaluation.false_tracks == 1


def test_frames_ignored_as_unstable_leave_the_visible_frames_instead_of_counting_as_missed() -> None:
    bat = bat_along(0, range(10, 30))
    detections = detections_on(bat)
    detections[20] = detections[20] + [detection(20, 20 + 10 * i, 300) for i in range(40)]

    result = evaluate(make_run([bat], detections), TrackConfig(), MATCH, (), StabilityConfig(pad_s=0.1)).bats[0]

    assert result.visible_frames == 20 - 7


def test_a_track_born_in_a_crowded_burst_is_not_counted_as_a_false_track() -> None:
    bat = bat_along(0, range(10, 30))
    burst = {
        f: [detection(f, 60 + 12 * (f - 50), 300)] + [detection(f, 20 + 15 * i, 100) for i in range(30)]
        for f in range(50, 60)
    }
    run = make_run([bat], detections_on(bat) | burst)

    assert evaluate(run, TrackConfig(), MATCH, (), StabilityConfig()).false_tracks == 0
    assert evaluate(run, TrackConfig(), MATCH, (), StabilityConfig(max_blobs=0)).false_tracks == 1
