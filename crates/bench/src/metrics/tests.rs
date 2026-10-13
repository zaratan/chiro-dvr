use std::num::NonZeroU32;

use engine::settings::Region;
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;
use types::detection::Detection;
use types::frame::FrameIndex;
use types::span::Span;

use super::{Judging, evaluate, is_visible};
use crate::config::MatchConfig;
use crate::synthetic::Point;
use crate::synthetic::injection::Observation;
use crate::testing::{along, detection, observed, on, run, scene};

const Y: f64 = 180.0;

fn settings() -> Judging {
    Judging {
        track: TrackConfig::default(),
        stability: StabilityConfig::default(),
        matching: MatchConfig::new(12.0, 0.5).unwrap(),
    }
}

fn line(first: usize, count: usize, start: (f64, f64)) -> Vec<Detection> {
    (0..count)
        .map(|offset| detection(first + offset, start.0 + 10.0 * offset as f64, start.1))
        .collect()
}

fn without(detections: Vec<Detection>, skipped: &[usize]) -> Vec<Detection> {
    detections
        .into_iter()
        .filter(|detection| !skipped.contains(&detection.frame.get()))
        .collect()
}

fn observation(x: f64, y: f64, effective: f64) -> Observation {
    Observation {
        frame: FrameIndex::new(0),
        at: Point { x, y },
        effective,
    }
}

#[test]
fn a_target_under_a_display_region_on_the_frame_edge_or_without_contrast_is_not_visible() {
    let band = [Region {
        x0: 0.0,
        y0: 0.0,
        x1: 1.0,
        y1: 0.1,
    }];

    assert!(is_visible(&observation(240.0, 180.0, -40.0), 3.0, scene(), &band));
    assert!(!is_visible(&observation(240.0, 5.0, -40.0), 3.0, scene(), &band));
    assert!(!is_visible(&observation(2.0, 180.0, -40.0), 3.0, scene(), &[]));
    assert!(is_visible(&observation(6.0, 354.0, -8.0), 3.0, scene(), &[]));
    assert!(!is_visible(&observation(240.0, 180.0, -7.9), 3.0, scene(), &[]));
}

#[test]
fn a_fully_detected_target_is_complete_with_no_delay() {
    let target = along(0, 10, 20, Y);
    let detections = on(&target);

    let evaluation = evaluate(
        &run(vec![observed(target, -40.0)], detections, Vec::new(), Vec::new()),
        settings(),
    )
    .unwrap();

    let result = evaluation.targets[0];
    assert!(result.found);
    assert_eq!(result.completeness, 1.0);
    assert_eq!(
        (result.start_delay, result.end_early, result.fragments),
        (Some(0), Some(0), 1)
    );
    assert_eq!((result.visible_frames, result.median_effective), (20, -40.0));
    assert_eq!(
        (
            evaluation.injected_tracks,
            evaluation.false_tracks,
            evaluation.not_visible
        ),
        (1, 0, 0)
    );
}

#[test]
fn a_late_start_and_an_early_end_are_counted_in_frames() {
    let target = along(0, 10, 20, Y);
    let detections = without(on(&target), &[10, 11, 12, 28, 29]);

    let result = evaluate(
        &run(vec![observed(target, -40.0)], detections, Vec::new(), Vec::new()),
        settings(),
    )
    .unwrap()
    .targets[0];

    assert_eq!((result.start_delay, result.end_early), (Some(3), Some(2)));
    assert_eq!(result.completeness, 15.0 / 20.0);
}

#[test]
fn a_target_missed_is_visible_but_not_found_and_has_no_delay() {
    let target = along(0, 10, 20, Y);

    let result = evaluate(
        &run(vec![observed(target, -40.0)], Vec::new(), Vec::new(), Vec::new()),
        settings(),
    )
    .unwrap()
    .targets[0];

    assert!(!result.found);
    assert_eq!(
        (
            result.completeness,
            result.start_delay,
            result.end_early,
            result.fragments
        ),
        (0.0, None, None, 0)
    );
}

#[test]
fn a_target_on_a_saturated_sky_is_left_out_rather_than_missed() {
    let target = along(0, 10, 20, Y);

    let evaluation = evaluate(
        &run(vec![observed(target, 0.0)], Vec::new(), Vec::new(), Vec::new()),
        settings(),
    )
    .unwrap();

    assert_eq!(evaluation.targets, []);
    assert_eq!(evaluation.not_visible, 1);
}

#[test]
fn a_track_split_by_a_long_gap_counts_as_two_fragments() {
    let target = along(0, 10, 30, Y);
    let detections = without(on(&target), &(20..28).collect::<Vec<_>>());

    let result = evaluate(
        &run(vec![observed(target, -40.0)], detections, Vec::new(), Vec::new()),
        settings(),
    )
    .unwrap()
    .targets[0];

    assert_eq!(result.fragments, 2);
}

#[test]
fn a_track_of_no_target_and_no_reference_track_is_a_false_track_and_one_on_a_reference_track_is_not() {
    let target = along(0, 10, 20, Y);
    let stray = line(50, 10, (60.0, 300.0));
    let both = on(&target).into_iter().chain(stray.clone()).collect::<Vec<_>>();

    let alone = evaluate(
        &run(
            vec![observed(target.clone(), -40.0)],
            both.clone(),
            Vec::new(),
            Vec::new(),
        ),
        settings(),
    )
    .unwrap();
    let known = evaluate(&run(vec![observed(target, -40.0)], both, stray, Vec::new()), settings()).unwrap();

    assert_eq!((alone.false_tracks, alone.reference_tracks), (1, 0));
    assert_eq!((known.false_tracks, known.reference_tracks), (0, 1));
}

#[test]
fn frames_ignored_as_unstable_leave_the_visible_frames_instead_of_counting_as_missed() {
    let target = along(0, 10, 20, Y);
    let crowd = (0..40).map(|index| detection(20, 20.0 + 10.0 * f64::from(index), 300.0));
    let detections = on(&target).into_iter().chain(crowd).collect();
    let padded = Judging {
        stability: StabilityConfig::new(NonZeroU32::new(20), 0.1).unwrap(),
        ..settings()
    };

    let result = evaluate(
        &run(vec![observed(target, -40.0)], detections, Vec::new(), Vec::new()),
        padded,
    )
    .unwrap()
    .targets[0];

    assert_eq!(result.visible_frames, 20 - 7);
}

#[test]
fn a_track_born_in_a_crowded_burst_is_not_a_false_track_unless_the_stability_check_is_off() {
    let target = along(0, 10, 20, Y);
    let burst = (50..60).flat_map(|frame| {
        std::iter::once(detection(frame, 60.0 + 12.0 * (frame - 50) as f64, 300.0))
            .chain((0..30).map(move |index| detection(frame, 20.0 + 15.0 * f64::from(index), 100.0)))
    });
    let detections: Vec<Detection> = on(&target).into_iter().chain(burst).collect();
    let crowded = run(vec![observed(target, -40.0)], detections, Vec::new(), Vec::new());
    let unchecked = Judging {
        stability: StabilityConfig::new(None, 1.0).unwrap(),
        ..settings()
    };

    assert_eq!(evaluate(&crowded, settings()).unwrap().false_tracks, 0);
    assert_eq!(evaluate(&crowded, unchecked).unwrap().false_tracks, 1);
}

#[test]
fn damaged_frames_are_ignored_in_the_injected_and_the_reference_runs_like_the_command() {
    let damaged = vec![Span::new(FrameIndex::new(30), FrameIndex::new(44)).unwrap()];
    let target = along(0, 20, 20, Y);
    let flight_in_damage = line(31, 12, (300.0, 300.0));
    let detections = on(&target).into_iter().chain(flight_in_damage.clone()).collect();

    let evaluation = evaluate(
        &run(
            vec![observed(target, -40.0)],
            detections,
            flight_in_damage,
            damaged.clone(),
        ),
        settings(),
    )
    .unwrap();
    let inside = evaluate(
        &run(
            vec![observed(along(1, 31, 9, Y), -40.0)],
            Vec::new(),
            Vec::new(),
            damaged,
        ),
        settings(),
    )
    .unwrap();

    assert_eq!(
        (evaluation.targets[0].visible_frames, evaluation.targets[0].completeness),
        (10, 1.0)
    );
    assert_eq!(
        (
            evaluation.injected_tracks,
            evaluation.reference_tracks,
            evaluation.false_tracks
        ),
        (1, 0, 0)
    );
    assert_eq!((inside.targets.len(), inside.not_visible), (0, 1));
}

#[test]
fn a_target_seen_on_fewer_frames_than_the_minimum_hits_is_not_visible() {
    let target = along(0, 10, 5, Y);

    let evaluation = evaluate(
        &run(vec![observed(target, -40.0)], Vec::new(), Vec::new(), Vec::new()),
        settings(),
    )
    .unwrap();

    assert_eq!(evaluation.not_visible, 1);
}
