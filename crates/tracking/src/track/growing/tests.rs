use types::frame::FrameIndex;

use super::Growing;
use crate::fake::detection;

fn growing(points: &[(usize, f64)]) -> Growing {
    let mut points = points.iter().map(|&(frame, x)| detection(frame, x, 0.0));
    let mut track = Growing::new(points.next().unwrap());
    for point in points {
        track.push(point);
    }
    track
}

#[test]
fn a_single_point_predicts_its_own_position() {
    let track = growing(&[(4, 12.0)]);
    assert_eq!(track.predicted(FrameIndex::new(5)), (12.0, 0.0));
    assert!(!track.has_velocity());
}

#[test]
fn saccade_prediction_error_stays_under_a_third_of_the_long_step() {
    let xs = [
        0.0, 20.0, 30.0, 40.0, 60.0, 70.0, 80.0, 100.0, 110.0, 120.0, 140.0, 150.0,
    ];
    let points: Vec<(usize, f64)> = xs.iter().copied().enumerate().collect();
    let worst = (3..xs.len())
        .map(|known| (growing(&points[..known]).predicted(FrameIndex::new(known)).0 - xs[known]).abs())
        .fold(0.0, f64::max);
    assert!(worst <= 20.0 / 3.0 + 1e-9);
}

#[test]
fn prediction_falls_back_to_the_last_two_points_on_a_young_track() {
    let track = growing(&[(0, 0.0), (1, 10.0)]);
    assert_eq!(track.predicted(FrameIndex::new(2)), (20.0, 0.0));
    assert!(track.has_velocity());
}

#[test]
fn prediction_spans_three_frames_even_across_a_gap() {
    let track = growing(&[(0, 0.0), (1, 10.0), (2, 20.0), (5, 80.0), (6, 90.0)]);
    let (x, y) = track.predicted(FrameIndex::new(7));
    assert!((x - (90.0 + (90.0 - 20.0) / 4.0)).abs() < 1e-9);
    assert_eq!(y, 0.0);
}

#[test]
fn the_points_come_back_in_frame_order() {
    let frames: Vec<usize> = growing(&[(0, 0.0), (2, 1.0), (3, 2.0)])
        .into_points()
        .iter()
        .map(|point| point.frame.get())
        .collect();
    assert_eq!(frames, [0, 2, 3]);
}
