use types::frame::FrameIndex;

use super::Association;
use crate::fake::detection;
use crate::track::growing::Growing;

#[test]
fn an_established_track_keeps_its_detection_over_a_closer_newborn() {
    let mut established = Growing::new(detection(0, 0.0, 0.0));
    established.push(detection(1, 10.0, 0.0));
    let newborn = Growing::new(detection(1, 21.0, 0.0));
    let mut active = vec![newborn, established];
    let found = [detection(2, 20.0, 0.0)];
    let unmatched: Vec<_> = Association::default()
        .assign(&mut active, &found, FrameIndex::new(2), 120.0)
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
        association.assign(&mut active, &found, FrameIndex::new(1), 5.0).count(),
        0
    );
    let mut active = vec![Growing::new(detection(0, 0.0, 0.0))];
    assert_eq!(
        association
            .assign(&mut active, &found, FrameIndex::new(1), 4.999)
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
        .assign(&mut active, &found, FrameIndex::new(1), 120.0)
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
        .assign(&mut active, &found, FrameIndex::new(1), 120.0)
        .map(|detection| detection.x)
        .collect();
    assert_eq!(unmatched, [900.0, 500.0]);
}

#[test]
fn at_equal_distance_the_detection_listed_first_is_taken() {
    let mut active = vec![Growing::new(detection(0, 10.0, 0.0))];
    let found = [detection(1, 15.0, 0.0), detection(1, 5.0, 0.0)];
    let unmatched: Vec<f64> = Association::default()
        .assign(&mut active, &found, FrameIndex::new(1), 120.0)
        .map(|detection| detection.x)
        .collect();
    assert_eq!(unmatched, [5.0]);
    assert!((active[0].last().x - 15.0).abs() < 1e-12);
}
