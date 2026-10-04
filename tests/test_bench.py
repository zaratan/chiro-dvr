from __future__ import annotations

import json
from pathlib import Path

import pytest

from batdetect.bench import (
    BenchRun,
    MatchConfig,
    Observation,
    assign_tracks,
    cache_key,
    evaluate,
    is_visible,
    main,
    wilson,
)
from batdetect.pipeline import DetectConfig, Detection, Region, Track, TrackConfig, VideoInfo
from batdetect.synthetic import BatClass, Sampling, SyntheticBat
from helpers import background, detection, write_video

INFO = VideoInfo(fps=30.0, frame_count=100, width=480, height=360, work_width=160, work_height=120)
MATCH = MatchConfig(radius=12, purity=0.5)


def bat_along(bat_id: int, frames: range, y: float = 60) -> SyntheticBat:
    positions = tuple((90.0 + 18 * k, y * INFO.scale) for k in range(len(frames)))
    return SyntheticBat(bat_id, frames.start, positions, tuple(-40.0 for _ in frames), -40.0, 3.0)


def observations_of(bat: SyntheticBat, effective: float = -40) -> list[Observation]:
    return [Observation(bat.start_frame + k, x, y, effective) for k, (x, y) in enumerate(bat.positions)]


def detections_on(bat: SyntheticBat, skip: set[int] | None = None) -> dict[int, list[Detection]]:
    out: dict[int, list[Detection]] = {}
    for k, (x, y) in enumerate(bat.positions):
        frame = bat.start_frame + k
        if frame not in (skip or set()):
            out[frame] = [detection(frame, x, y)]
    return out


def make_run(bats: list[SyntheticBat], injected: dict[int, list[Detection]], effective: float = -40) -> BenchRun:
    frames = {f: injected.get(f, []) for f in range(INFO.frame_count)}
    observations = {b.id: observations_of(b, effective) for b in bats}
    return BenchRun("k", INFO, {f: [] for f in range(INFO.frame_count)}, frames, bats, observations)


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


def test_track_mostly_elsewhere_is_not_credited_to_a_target_it_brushes() -> None:
    bat = bat_along(0, range(10, 30))
    brush = [detection(10, 30, 60)] + [detection(f, 30 + 18 * (f - 10), 300) for f in range(11, 30)]
    track = Track(1, brush)
    truth = {o.frame: [(0, o.x, o.y)] for o in observations_of(bat)}

    assert assign_tracks([track], truth, MATCH) == {}


def test_track_with_no_target_and_no_reference_counterpart_is_a_false_track() -> None:
    bat = bat_along(0, range(10, 30))
    stray = {f: [detection(f, 60 + 12 * (f - 50), 300)] for f in range(50, 60)}

    evaluation = evaluate(make_run([bat], detections_on(bat) | stray), TrackConfig(), MATCH)

    assert evaluation.false_tracks == 1


def test_cache_key_ignores_int_versus_float_spelling(tmp_path: Path) -> None:
    video = tmp_path / "v.mp4"
    video.write_bytes(b"x")
    sampling = Sampling(1, (BatClass(-30, 2),), 3, 72)

    assert cache_key(video, DetectConfig(min_area=4), sampling) == cache_key(
        video, DetectConfig(min_area=4.0), sampling
    )


def test_wilson_interval_matches_the_textbook_value() -> None:
    low, high = wilson(5, 10)

    assert (round(low, 3), round(high, 3)) == (0.237, 0.763)


def test_bench_command_writes_a_report_on_a_synthetic_video(tmp_path: Path) -> None:
    video = tmp_path / "plain.mp4"
    write_video(video, [background(320, 240, seed=i) for i in range(150)], fps=30)
    out = tmp_path / "bench"

    code = main([str(video), "-o", str(out), "--per-class", "3", "--amplitudes", "-80", "--sigmas", "4"])

    report = json.loads((out / "plain" / "bench.json").read_text())
    assert code == 0
    assert report["summary"][0]["n"] + report["not_visible"] == 3
    assert report["summary"][0]["found"] >= 1
