use types::frame::FrameIndex;

use super::{Flight, bezier, bezier_length, passing_target, saccade_progress};
use crate::random::Xoshiro256;
use crate::synthetic::{Class, Motion, Point};

fn steps(progress: &[f64]) -> Vec<f64> {
    progress.windows(2).map(|pair| pair[1] - pair[0]).collect()
}

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() < 1e-12
}

#[test]
fn the_saccade_doubles_one_step_in_three_and_goes_from_zero_to_one() {
    let progress = saccade_progress(10, 0);
    let steps = steps(&progress);

    assert_eq!(progress.len(), 10);
    assert_eq!(progress[0], 0.0);
    assert!(close(progress[9], 1.0));
    assert!(close(steps[2], 2.0 * steps[0]));
    assert!(close(steps[5], 2.0 * steps[3]));
    assert!(steps.iter().all(|&step| step > 0.0));
}

#[test]
fn the_phase_moves_the_doubled_step() {
    let steps = steps(&saccade_progress(10, 1));

    assert!(close(steps[1], 2.0 * steps[0]));
    assert!(close(steps[4], 2.0 * steps[3]));
}

#[test]
fn a_single_frame_stays_at_the_start() {
    assert_eq!(saccade_progress(1, 2), [0.0]);
}

#[test]
fn a_curve_starts_and_ends_on_its_end_points_and_bends_towards_its_control() {
    let controls = [
        Point { x: 0.0, y: 0.0 },
        Point { x: 50.0, y: 100.0 },
        Point { x: 100.0, y: 0.0 },
    ];

    assert_eq!(bezier(controls, 0.0), controls[0]);
    assert_eq!(bezier(controls, 1.0), controls[2]);
    assert_eq!(bezier(controls, 0.5), Point { x: 50.0, y: 50.0 });
}

#[test]
fn the_length_of_a_straight_curve_is_the_distance_between_its_ends() {
    let controls = [
        Point { x: 0.0, y: 0.0 },
        Point { x: 30.0, y: 40.0 },
        Point { x: 60.0, y: 80.0 },
    ];

    assert!(close(bezier_length(controls), 100.0));
}

#[test]
fn a_passing_target_flies_from_entry_to_exit_and_accelerates() {
    let flight = Flight {
        controls: [
            Point { x: 0.0, y: 0.0 },
            Point { x: 50.0, y: 0.0 },
            Point { x: 100.0, y: 0.0 },
        ],
        speed: 5.0,
        acceleration: 1.8,
    };
    let class = Class::new(-40.0, 2.0, Motion::Pass).unwrap();

    let target = passing_target(3, FrameIndex::new(10), flight, class, &mut Xoshiro256::seeded(0));
    let positions = target.positions();
    let last = positions.len() - 1;

    assert!((20..=21).contains(&positions.len()));
    assert_eq!(positions[0], Point { x: 0.0, y: 0.0 });
    assert!(positions[last].distance(Point { x: 100.0, y: 0.0 }) < 1e-9);
    assert!(positions[last].x - positions[last - 3].x > 2.0 * (positions[3].x - positions[0].x));
    assert_eq!(target.position(FrameIndex::new(9)), None);
    assert_eq!(target.position(FrameIndex::new(10)), Some(positions[0]));
    assert_eq!(target.position(FrameIndex::new(11 + last)), None);
    assert_eq!(target.end(), FrameIndex::new(10 + last));
}

#[test]
fn a_passing_target_lasts_at_least_two_frames_and_flickers_between_sixty_and_a_hundred_percent() {
    let flight = Flight {
        controls: [
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 2.0, y: 0.0 },
        ],
        speed: 60.0,
        acceleration: 1.0,
    };
    let class = Class::new(-30.0, 3.0, Motion::Pass).unwrap();

    let target = passing_target(0, FrameIndex::new(0), flight, class, &mut Xoshiro256::seeded(5));

    assert_eq!(target.positions().len(), 2);
    assert!(
        target
            .amplitudes()
            .iter()
            .all(|&amplitude| (-30.0..=-18.0).contains(&amplitude))
    );
}
