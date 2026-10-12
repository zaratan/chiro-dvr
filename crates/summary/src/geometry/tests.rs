use super::Trajectory;
use crate::testing::{path, track};

fn near(found: [f64; 2], expected: [f64; 2]) -> bool {
    (found[0] - expected[0]).abs() < 1e-9 && (found[1] - expected[1]).abs() < 1e-9
}

#[test]
fn track_points_keep_original_pixel_coordinates() {
    let walked = track(1, 0, &[[10.5, 20.0], [12.5, 20.0], [14.5, 20.0]]);
    assert_eq!(
        Trajectory::of(&walked).points(),
        [[10.5, 20.0], [12.5, 20.0], [14.5, 20.0]]
    );
}

#[test]
fn cumulative_length_adds_segment_lengths() {
    assert_eq!(path(&[[0.0, 0.0], [3.0, 4.0], [3.0, 10.0]]).total(), 11.0);
}

#[test]
fn point_at_length_follows_distance_not_point_count() {
    let slow_start = path(&[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], [100.0, 0.0]]);
    assert!(near(slow_start.at(50.0), [50.0, 0.0]));
}

#[test]
fn single_point_path_has_zero_length_and_stays_put() {
    let lone = path(&[[7.0, 9.0]]);
    assert_eq!(lone.total(), 0.0);
    assert_eq!(lone.at(5.0), [7.0, 9.0]);
    assert_eq!(lone.end_direction(10.0), None);
}

#[test]
fn resample_spaces_points_evenly_and_keeps_the_end() {
    assert_eq!(
        path(&[[0.0, 0.0], [10.0, 0.0]]).resampled(4.0),
        [[0.0, 0.0], [4.0, 0.0], [8.0, 0.0], [10.0, 0.0]]
    );
}

#[test]
fn end_direction_looks_back_over_the_span_not_the_last_jitter() {
    let [dx, dy] = path(&[[0.0, 0.0], [100.0, 0.0], [100.0, 1.0]])
        .end_direction(50.0)
        .unwrap();
    assert!((dx - 1.0).abs() < 0.01, "{dx}");
    assert!((dy - 0.02).abs() < 0.01, "{dy}");
}

#[test]
fn a_point_repeated_in_place_does_not_break_the_walk_along_the_path() {
    let paused = path(&[[0.0, 0.0], [10.0, 0.0], [10.0, 0.0], [20.0, 0.0]]);
    assert!(near(paused.at(10.0), [10.0, 0.0]));
    assert!(near(paused.at(15.0), [15.0, 0.0]));
    assert!(paused.resampled(5.0).iter().all(|spot| spot[0].is_finite()));
}

#[test]
fn a_step_that_is_not_positive_keeps_only_the_end() {
    let segment = path(&[[0.0, 0.0], [10.0, 0.0]]);
    assert_eq!(segment.resampled(0.0), [[10.0, 0.0]]);
    assert_eq!(segment.resampled(f64::NAN), [[10.0, 0.0]]);
}

#[test]
fn the_nearest_point_of_a_path_lies_between_its_points_not_only_on_them() {
    let corner = path(&[[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]]);
    assert!(near(corner.nearest([40.0, 30.0]), [40.0, 0.0]));
    assert!(near(corner.nearest([130.0, 60.0]), [100.0, 60.0]));
    assert!(near(path(&[[7.0, 9.0]]).nearest([0.0, 0.0]), [7.0, 9.0]));
}
