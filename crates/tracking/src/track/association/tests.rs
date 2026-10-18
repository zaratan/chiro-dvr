use types::frame::FrameIndex;

use super::{Association, Gate};
use crate::fake::{detection, size, weak};
use crate::track::growing::Growing;
use crate::track::reach::Reach;

fn jump(max_jump: f64) -> Gate {
    Gate {
        max_jump,
        reach: None,
        size: size(),
    }
}

#[test]
fn an_established_track_keeps_its_detection_over_a_closer_newborn() {
    let mut established = Growing::new(detection(0, 0.0, 0.0));
    established.push(detection(1, 10.0, 0.0));
    let newborn = Growing::new(detection(1, 21.0, 0.0));
    let mut active = vec![newborn, established];
    let found = [detection(2, 20.0, 0.0)];
    let unmatched: Vec<_> = Association::default()
        .assign(&mut active, &found, FrameIndex::new(2), jump(120.0))
        .collect();
    assert_eq!(unmatched, []);
    assert_eq!(active[1].last().frame.get(), 2);
    assert_eq!(active[0].last().frame.get(), 1);
}

#[test]
fn a_detection_exactly_max_jump_away_is_taken() {
    let mut active = vec![Growing::new(detection(0, 0.0, 0.0))];
    let found = [detection(1, 3.0, 4.0)];
    let mut association = Association::default();
    assert_eq!(
        association
            .assign(&mut active, &found, FrameIndex::new(1), jump(5.0))
            .count(),
        0
    );
    let mut active = vec![Growing::new(detection(0, 0.0, 0.0))];
    assert_eq!(
        association
            .assign(&mut active, &found, FrameIndex::new(1), jump(4.999))
            .count(),
        1
    );
}

#[test]
fn at_equal_distance_the_track_created_first_wins() {
    let mut active = vec![
        Growing::new(detection(0, 0.0, 0.0)),
        Growing::new(detection(0, 10.0, 0.0)),
    ];
    let found = [detection(1, 5.0, 0.0)];
    let unmatched = Association::default()
        .assign(&mut active, &found, FrameIndex::new(1), jump(120.0))
        .count();
    assert_eq!(unmatched, 0);
    assert_eq!((active[0].last().frame.get(), active[1].last().frame.get()), (1, 0));
}

#[test]
fn unmatched_detections_come_back_in_their_order() {
    let mut active = vec![Growing::new(detection(0, 0.0, 0.0))];
    let found = [
        detection(1, 900.0, 0.0),
        detection(1, 1.0, 0.0),
        detection(1, 500.0, 0.0),
    ];
    let unmatched: Vec<f64> = Association::default()
        .assign(&mut active, &found, FrameIndex::new(1), jump(120.0))
        .map(|detection| detection.x)
        .collect();
    assert_eq!(unmatched, [900.0, 500.0]);
}

#[test]
fn at_equal_distance_the_detection_listed_first_is_taken() {
    let mut active = vec![Growing::new(detection(0, 10.0, 0.0))];
    let found = [detection(1, 15.0, 0.0), detection(1, 5.0, 0.0)];
    let unmatched: Vec<f64> = Association::default()
        .assign(&mut active, &found, FrameIndex::new(1), jump(120.0))
        .map(|detection| detection.x)
        .collect();
    assert_eq!(unmatched, [5.0]);
    assert!((active[0].last().x - 15.0).abs() < 1e-12);
}

#[test]
fn a_track_takes_a_strong_detection_over_a_closer_weak_one() {
    let mut active = vec![Growing::new(detection(0, 0.0, 0.0))];
    let found = [weak(1, 12.0, 0.0), detection(1, 21.0, 0.0)];
    let unmatched: Vec<_> = Association::default()
        .assign(&mut active, &found, FrameIndex::new(1), jump(120.0))
        .collect();
    assert_eq!(active[0].last(), &found[1]);
    assert_eq!(unmatched, [found[0]]);
}

#[test]
fn a_track_without_a_strong_detection_in_reach_takes_a_weak_one() {
    let mut active = vec![Growing::new(detection(0, 0.0, 0.0))];
    let found = [detection(1, 500.0, 0.0), weak(1, 12.0, 0.0)];
    let unmatched: Vec<_> = Association::default()
        .assign(&mut active, &found, FrameIndex::new(1), jump(120.0))
        .collect();
    assert_eq!(active[0].last(), &found[1]);
    assert_eq!(unmatched, [found[0]]);
}

#[test]
fn a_newborn_track_without_a_speed_is_held_by_max_jump_alone() {
    let mut active = vec![Growing::new(detection(0, 300.0, 500.0))];
    let found = [detection(1, 410.0, 500.0)];
    let gate = Gate {
        reach: Some(Reach::default()),
        ..jump(120.0)
    };
    let unmatched = Association::default()
        .assign(&mut active, &found, FrameIndex::new(1), gate)
        .count();
    assert_eq!(unmatched, 0);
}

#[test]
fn a_detection_far_beyond_the_reach_of_a_slow_track_is_left_to_open_its_own() {
    let mut track = Growing::new(detection(0, 300.0, 500.0));
    track.push(detection(1, 305.0, 500.0));
    let mut active = vec![track];
    let found = [detection(2, 310.0, 570.0)];
    let gate = Gate {
        reach: Some(Reach::default()),
        ..jump(120.0)
    };
    let unmatched = Association::default()
        .assign(&mut active, &found, FrameIndex::new(2), gate)
        .count();
    assert_eq!(unmatched, 1);
}
