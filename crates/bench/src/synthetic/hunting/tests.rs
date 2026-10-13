use std::f64::consts::{FRAC_PI_4, PI, TAU};
use std::num::NonZeroU32;

use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::size::SourceSize;

use super::{CRUISE_SPEED, MAX_DURATION_S, STRAIGHT_DRIFT, hunting_path, hunting_target, turn_profile, with_saccade};
use crate::random::Xoshiro256;
use crate::synthetic::{Class, Motion, Point, Scene};

const SEEDS: std::ops::Range<u64> = 0..40;

fn scene() -> Scene {
    Scene::new(
        SourceSize::new(NonZeroU32::new(1440).unwrap(), NonZeroU32::new(1080).unwrap()),
        FrameRate::new(Rational::new(30, 1).unwrap()).unwrap(),
    )
}

fn hunter(motion: Motion) -> Class {
    Class::new(-60.0, 3.0, motion).unwrap()
}

fn headings(points: &[Point]) -> Vec<f64> {
    points
        .windows(2)
        .map(|pair| (pair[1].y - pair[0].y).atan2(pair[1].x - pair[0].x))
        .collect()
}

fn turns(points: &[Point]) -> Vec<f64> {
    headings(points)
        .windows(2)
        .map(|pair| ((pair[1] - pair[0] + PI).rem_euclid(TAU) - PI).abs())
        .collect()
}

fn sharpest_turn(points: &[Point]) -> f64 {
    let mut turned = 0.0;
    let unwrapped: Vec<f64> = std::iter::once(0.0)
        .chain(headings(points).windows(2).map(|pair| {
            turned += (pair[1] - pair[0] + PI).rem_euclid(TAU) - PI;
            turned
        }))
        .collect();
    let span = 20.min(unwrapped.len() - 1);
    unwrapped
        .windows(span + 1)
        .map(|window| (window[span] - window[0]).abs())
        .fold(0.0, f64::max)
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        f64::midpoint(values[middle - 1], values[middle])
    } else {
        values[middle]
    }
}

#[test]
fn a_turn_turns_by_the_requested_angle_and_is_tightest_at_its_apex() {
    let turn = turn_profile(20.0, 0.25, 0.3, PI);
    let slowest = (0..turn.len())
        .min_by(|&a, &b| turn[a].speed.total_cmp(&turn[b].speed))
        .unwrap();
    let sharpest = (0..turn.len())
        .max_by(|&a, &b| turn[a].turn.total_cmp(&turn[b].turn))
        .unwrap();

    assert!((turn.iter().map(|step| step.turn).sum::<f64>() - PI).abs() < 1e-12);
    assert!(turn[slowest].speed < 0.3 * 20.0);
    assert_eq!(sharpest, slowest);
}

#[test]
fn a_turn_too_wide_for_sixty_frames_stops_at_sixty_frames_short_of_its_angle() {
    let turn = turn_profile(20.0, 0.5, 0.15, 100.0);

    assert_eq!(turn.len(), 60);
    assert!(turn.iter().map(|step| step.turn).sum::<f64>() < 100.0);
}

#[test]
fn every_hunting_path_enters_from_outside_heading_inward_and_really_turns() {
    let center = Point { x: 720.0, y: 540.0 };
    for seed in SEEDS {
        let path = hunting_path(&mut Xoshiro256::seeded(seed), hunter(Motion::Hunt), scene());
        let start = path[0];

        assert!(
            !(0.0 < start.x && start.x < 1440.0 && 0.0 < start.y && start.y < 1080.0),
            "{seed}"
        );
        assert!(path[1].distance(center) < start.distance(center), "{seed}");
        assert!(sharpest_turn(&path) >= FRAC_PI_4, "{seed}");
    }
}

#[test]
fn hunting_paths_slow_down_in_turns_and_stop_at_the_time_limit_or_out_of_the_frame() {
    let limit = (MAX_DURATION_S * 30.0).round() as usize;
    let margin = 3.0 * 3.0 + 2.0;
    for seed in SEEDS {
        let path = hunting_path(&mut Xoshiro256::seeded(seed), hunter(Motion::Hunt), scene());
        let steps: Vec<f64> = path.windows(2).map(|pair| pair[0].distance(pair[1])).collect();
        let fastest = steps.iter().copied().fold(0.0, f64::max);
        let slowest = steps.iter().copied().fold(f64::INFINITY, f64::min);
        let end = path[path.len() - 1];
        let gone = !((-margin..=1440.0 + margin).contains(&end.x) && (-margin..=1080.0 + margin).contains(&end.y));

        assert!(fastest <= CRUISE_SPEED.1, "{seed}");
        assert!(slowest < 0.6 * fastest, "{seed}");
        assert!(path.len() == limit || (path.len() < limit && gone), "{seed}");
    }
}

#[test]
fn circling_spends_more_time_turning_than_hunting() {
    let turning_share = |motion| {
        median(
            SEEDS
                .map(|seed| {
                    let turns = turns(&hunting_path(&mut Xoshiro256::seeded(seed), hunter(motion), scene()));
                    turns.iter().filter(|&&turn| turn > STRAIGHT_DRIFT).count() as f64 / turns.len() as f64
                })
                .collect(),
        )
    };

    assert!(turning_share(Motion::Circle) > 0.5);
    assert!(turning_share(Motion::Circle) > turning_share(Motion::Hunt));
}

#[test]
fn the_saccade_keeps_the_ends_and_doubles_one_step_in_three() {
    let path: Vec<Point> = (0..13)
        .map(|x| Point {
            x: f64::from(x),
            y: 0.0,
        })
        .collect();

    let resampled = with_saccade(&path, 0);
    let steps: Vec<f64> = resampled.windows(2).map(|pair| pair[1].x - pair[0].x).collect();

    assert_eq!(resampled.len(), 13);
    assert_eq!(resampled[0], path[0]);
    assert!(resampled[12].distance(path[12]) < 1e-12);
    assert!((steps[2] - 2.0 * steps[0]).abs() < 1e-12);
    assert!((steps[5] - 2.0 * steps[3]).abs() < 1e-12);
}

#[test]
fn a_single_point_path_stays_where_it_is() {
    let path = [Point { x: 4.0, y: 5.0 }];

    assert_eq!(with_saccade(&path, 1), path);
}

#[test]
fn a_hunting_target_is_reproducible_and_carries_its_class() {
    let class = hunter(Motion::Circle);

    let one = hunting_target(4, FrameIndex::new(100), class, scene(), &mut Xoshiro256::seeded(9));
    let two = hunting_target(4, FrameIndex::new(100), class, scene(), &mut Xoshiro256::seeded(9));

    assert_eq!(one, two);
    assert_eq!(one.class(), class);
    assert_eq!(one.start(), FrameIndex::new(100));
    assert_eq!(one.amplitudes().len(), one.positions().len());
}
