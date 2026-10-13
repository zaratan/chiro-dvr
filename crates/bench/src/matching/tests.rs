use types::track::Track;

use super::{assign_tracks, matches_reference, near, reference_points, truth};
use crate::config::MatchConfig;
use crate::synthetic::Point;
use crate::testing::{along, detection, observed, on};

fn matching() -> MatchConfig {
    MatchConfig::new(12.0, 0.5).unwrap()
}

#[test]
fn a_detection_exactly_at_the_radius_is_near() {
    let at = Point { x: 100.0, y: 100.0 };

    assert!(near(&detection(0, 112.0, 100.0), at, 12.0));
    assert!(!near(&detection(0, 112.001, 100.0), at, 12.0));
}

#[test]
fn a_track_on_a_target_is_credited_to_it() {
    let target = along(0, 10, 20, 60.0);
    let track = Track::new(1, on(&target)).unwrap();

    assert_eq!(
        assign_tracks(&[track], &truth(&[observed(target, -40.0)]), matching()),
        [Some(0)]
    );
}

#[test]
fn a_track_mostly_elsewhere_is_not_credited_to_a_target_it_brushes() {
    let target = along(0, 10, 20, 60.0);
    let brush = std::iter::once(detection(10, 90.0, 60.0))
        .chain((11..30).map(|frame| detection(frame, 30.0 + 18.0 * (frame - 10) as f64, 300.0)))
        .collect();

    let assigned = assign_tracks(
        &[Track::new(1, brush).unwrap()],
        &truth(&[observed(target, -40.0)]),
        matching(),
    );

    assert_eq!(assigned, [None]);
}

#[test]
fn a_track_half_on_a_target_reaches_the_purity_and_one_point_less_does_not() {
    let target = along(0, 0, 10, 60.0);
    let half: Vec<_> = on(&target)
        .into_iter()
        .enumerate()
        .map(|(offset, point)| {
            if offset < 5 {
                point
            } else {
                detection(offset, point.x, 300.0)
            }
        })
        .collect();
    let mut less = half.clone();
    less[4].y = 300.0;
    let truth = truth(&[observed(target, -40.0)]);

    let assigned = assign_tracks(
        &[Track::new(1, half).unwrap(), Track::new(2, less).unwrap()],
        &truth,
        matching(),
    );

    assert_eq!(assigned, [Some(0), None]);
}

#[test]
fn a_track_crossing_two_targets_goes_to_the_one_it_covers_most_and_on_a_tie_to_the_first_met() {
    let first = along(0, 0, 4, 60.0);
    let second = along(1, 4, 6, 60.0);
    let longer = along(1, 4, 4, 60.0);
    let truth_most = truth(&[observed(first.clone(), -40.0), observed(second.clone(), -40.0)]);
    let truth_tie = truth(&[observed(first.clone(), -40.0), observed(longer.clone(), -40.0)]);
    let crossing: Vec<_> = on(&first).into_iter().chain(on(&second)).collect();
    let tied: Vec<_> = on(&first).into_iter().chain(on(&longer)).collect();

    assert_eq!(
        assign_tracks(&[Track::new(1, crossing).unwrap()], &truth_most, matching()),
        [Some(1)]
    );
    assert_eq!(
        assign_tracks(
            &[Track::new(1, tied).unwrap()],
            &truth_tie,
            MatchConfig::new(12.0, 0.4).unwrap()
        ),
        [Some(0)]
    );
}

#[test]
fn a_track_far_from_any_target_is_credited_to_none() {
    let target = along(0, 0, 10, 60.0);
    let away = Track::new(1, on(&along(5, 0, 10, 300.0))).unwrap();

    assert_eq!(
        assign_tracks(&[away], &truth(&[observed(target, -40.0)]), matching()),
        [None]
    );
}

#[test]
fn a_track_on_the_points_of_a_reference_track_matches_it_at_half_purity() {
    let reference = Track::new(1, on(&along(0, 0, 10, 60.0))).unwrap();
    let points = reference_points(std::slice::from_ref(&reference));
    let half: Vec<_> = reference
        .points()
        .iter()
        .enumerate()
        .map(|(offset, point)| {
            if offset < 5 {
                *point
            } else {
                detection(offset, point.x, 300.0)
            }
        })
        .collect();
    let elsewhere = Track::new(3, on(&along(0, 0, 10, 300.0))).unwrap();

    assert!(matches_reference(&reference, &points, matching()));
    assert!(matches_reference(&Track::new(2, half).unwrap(), &points, matching()));
    assert!(!matches_reference(&elsewhere, &points, matching()));
}
