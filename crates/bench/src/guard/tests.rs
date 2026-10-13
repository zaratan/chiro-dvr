use reference::bench::{BenchReport, ReportedTarget};
use serde_json::{Map, Value};
use types::frame::FrameIndex;

use super::{Change, Error, compare};
use crate::metrics::{Evaluation, TargetResult};
use crate::synthetic::{Class, Motion, Target};
use crate::testing::along;

fn class() -> Class {
    Class::new(-40.0, 3.0, Motion::Pass).unwrap()
}

fn result(id: usize, found: bool) -> TargetResult {
    TargetResult {
        id,
        class: class(),
        visible_frames: 20,
        found,
        completeness: if found { 1.0 } else { 0.0 },
        start_delay: found.then_some(0),
        end_early: found.then_some(0),
        fragments: usize::from(found),
        median_effective: -40.0,
    }
}

fn evaluation(found: &[(usize, bool)], false_tracks: usize) -> Evaluation {
    Evaluation {
        targets: found.iter().map(|&(id, found)| result(id, found)).collect(),
        not_visible: 0,
        false_tracks,
        reference_tracks: 14,
        injected_tracks: 20,
    }
}

fn reported(found: &[(usize, bool)], false_tracks: usize) -> BenchReport {
    BenchReport {
        video: "video.mp4".to_owned(),
        code: "v0.1.1".to_owned(),
        detect: Value::Null,
        track: Value::Null,
        stability: Value::Null,
        matching: Value::Null,
        sampling: Value::Null,
        summary: Vec::new(),
        not_visible: 0,
        false_tracks,
        reference_tracks: 14,
        injected_tracks: 20,
        bats: found
            .iter()
            .map(|&(id, found)| ReportedTarget {
                id,
                motion: "pass".to_owned(),
                amplitude: -40.0,
                sigma: 3.0,
                visible_frames: 20,
                found,
                completeness: 1.0,
                start_delay: None,
                end_early: None,
                fragments: 1,
                median_effective: -40.0,
            })
            .collect(),
        added: Map::new(),
    }
}

fn targets(count: usize) -> Vec<Target> {
    (0..count).map(|id| along(id, 10 * id, 20, 180.0)).collect()
}

fn all_found(count: usize) -> Vec<(usize, bool)> {
    (0..count).map(|id| (id, true)).collect()
}

#[test]
fn a_target_found_only_by_the_rust_is_listed_as_gained_and_never_fails() {
    let v011: Vec<_> = (0..10).map(|id| (id, id != 3)).collect();
    let judged = evaluation(&all_found(10), 0);

    let guard = compare(&judged, &judged, &reported(&v011, 0), &targets(10)).unwrap();

    assert_eq!(
        guard.classes[0].gained,
        [Change {
            id: 3,
            start: FrameIndex::new(30)
        }]
    );
    assert_eq!(
        (guard.classes[0].n, guard.classes[0].v011_found, guard.classes[0].found),
        (10, 9, 10)
    );
    assert!(!guard.failed());
}

#[test]
fn a_loss_above_the_lower_wilson_bound_passes_and_is_listed() {
    let rust: Vec<_> = (0..30).map(|id| (id, id != 7)).collect();
    let judged = evaluation(&rust, 0);

    let guard = compare(&judged, &judged, &reported(&all_found(30), 0), &targets(30)).unwrap();

    assert_eq!(
        guard.classes[0].lost,
        [Change {
            id: 7,
            start: FrameIndex::new(70)
        }]
    );
    assert!(guard.classes[0].lowest > 0.88 && guard.classes[0].lowest < 0.89);
    assert!(!guard.failed());
}

#[test]
fn a_proportion_found_below_the_lower_wilson_bound_fails() {
    let rust: Vec<_> = (0..30).map(|id| (id, id >= 5)).collect();
    let judged = evaluation(&rust, 0);

    let guard = compare(&judged, &judged, &reported(&all_found(30), 0), &targets(30)).unwrap();

    assert_eq!(guard.classes[0].lost.len(), 5);
    assert!(guard.classes[0].failed());
    assert!(guard.failed());
}

#[test]
fn a_target_visible_on_one_side_only_is_listed_apart_and_left_out_of_the_count() {
    let judged = evaluation(&[(0, true), (1, true), (3, false)], 2);

    let guard = compare(
        &judged,
        &judged,
        &reported(&[(0, true), (1, true), (2, true)], 1),
        &targets(4),
    )
    .unwrap();

    assert_eq!(
        guard.v011_only,
        [Change {
            id: 2,
            start: FrameIndex::new(20)
        }]
    );
    assert_eq!(
        guard.rust_only,
        [Change {
            id: 3,
            start: FrameIndex::new(30)
        }]
    );
    assert_eq!(guard.classes[0].n, 2);
    assert_eq!((guard.v011_false_tracks, guard.false_tracks), (1, 2));
}

#[test]
fn a_target_whose_visibility_changes_with_the_contrast_measured_by_the_rust_is_listed() {
    let judged = evaluation(&all_found(3), 0);
    let measured = evaluation(&[(0, true), (2, true)], 0);

    let guard = compare(&judged, &measured, &reported(&all_found(3), 0), &targets(3)).unwrap();

    assert_eq!(
        guard.contrast_differs,
        [Change {
            id: 1,
            start: FrameIndex::new(10)
        }]
    );
}

#[test]
fn a_recorded_target_that_was_not_replayed_or_of_unknown_motion_is_refused() {
    let judged = evaluation(&all_found(2), 0);
    let mut glide = reported(&all_found(2), 0);
    glide.bats[1].motion = "glide".to_owned();

    assert_eq!(
        compare(&judged, &judged, &reported(&all_found(2), 0), &targets(1)),
        Err(Error::Unknown { id: 1 })
    );
    assert!(matches!(
        compare(&judged, &judged, &glide, &targets(2)),
        Err(Error::Motion { id: 1, .. })
    ));
}

#[test]
fn classes_are_guarded_in_the_order_of_the_summary() {
    let judged = evaluation(&all_found(2), 0);
    let mut two = reported(&all_found(2), 0);
    two.bats[0].amplitude = -15.0;

    let guard = compare(&judged, &judged, &two, &targets(2)).unwrap();

    let amplitudes: Vec<f64> = guard.classes.iter().map(|guarded| guarded.class.amplitude()).collect();
    assert_eq!(amplitudes, [-40.0, -15.0]);
}
