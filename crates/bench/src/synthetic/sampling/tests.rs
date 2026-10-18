use std::num::{NonZeroU32, NonZeroUsize};

use proptest::prelude::*;
use types::detection::{Detection, Strength};
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::size::SourceSize;
use types::track::Track;

use super::{Occupancy, Sampling, random_targets};
use crate::synthetic::{Class, Motion, Point, Scene, Target};

const MIN_DISTANCE: f64 = 72.0;

fn scene() -> Scene {
    Scene::new(
        SourceSize::new(NonZeroU32::new(1440).unwrap(), NonZeroU32::new(1080).unwrap()),
        FrameRate::new(Rational::new(30, 1).unwrap()).unwrap(),
    )
}

fn sampling(seed: u64, motions: &[Motion], per_class: usize) -> Sampling {
    let classes = motions
        .iter()
        .flat_map(|&motion| {
            [(-60.0, 3.0), (-15.0, 1.5)].map(|(amplitude, sigma)| Class::new(amplitude, sigma, motion).unwrap())
        })
        .collect();
    Sampling {
        seed,
        classes,
        per_class: NonZeroUsize::new(per_class).unwrap(),
        min_distance: MIN_DISTANCE,
    }
}

fn crossing_track() -> Track {
    let points = (0..300)
        .map(|frame| Detection {
            frame: FrameIndex::new(frame),
            x: 4.8 * frame as f64,
            y: 540.0,
            left: 0.0,
            top: 0.0,
            width: 1.0,
            height: 1.0,
            area: 1.0,
            amplitude: -20.0,
            strength: Strength::Strong,
        })
        .collect();
    Track::new(0, points).unwrap()
}

fn points(target: &Target) -> impl Iterator<Item = (usize, Point)> + '_ {
    target
        .positions()
        .iter()
        .enumerate()
        .map(|(offset, &point)| (target.start().get() + offset, point))
}

#[test]
fn the_same_seed_places_the_same_targets() {
    let occupied = Occupancy::of_tracks(&[crossing_track()]);
    let sampling = sampling(3, &[Motion::Pass, Motion::Hunt], 4);

    let one = random_targets(&sampling, scene(), 900, &occupied);
    let two = random_targets(&sampling, scene(), 900, &occupied);

    assert_eq!(one, two);
    assert_eq!(one.len(), 16);
}

#[test]
fn targets_are_numbered_class_after_class_and_carry_their_class() {
    let sampling = sampling(1, &Motion::ALL, 3);

    let targets = random_targets(&sampling, scene(), 900, &Occupancy::default());

    assert_eq!(
        targets.iter().map(Target::id).collect::<Vec<_>>(),
        (0..18).collect::<Vec<_>>()
    );
    for target in &targets {
        assert_eq!(target.class(), sampling.classes[target.id() / 3]);
        assert!(target.end().get() < 900);
    }
}

#[test]
fn a_target_longer_than_the_video_is_not_placed() {
    let sampling = sampling(1, &[Motion::Pass], 5);

    assert_eq!(random_targets(&sampling, scene(), 2, &Occupancy::default()), []);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn no_target_comes_closer_than_the_minimum_distance_to_a_reference_track_or_another_target(seed: u64) {
        let track = crossing_track();
        let occupied = Occupancy::of_tracks(std::slice::from_ref(&track));

        let targets = random_targets(&sampling(seed, &Motion::ALL, 2), scene(), 900, &occupied);

        for (index, target) in targets.iter().enumerate() {
            for (frame, point) in points(target) {
                for reference in track.points().iter().filter(|reference| reference.frame.get() == frame) {
                    let near = Point { x: reference.x, y: reference.y };
                    prop_assert!(point.distance(near) >= MIN_DISTANCE);
                }
                for other in &targets[..index] {
                    if let Some(near) = other.position(FrameIndex::new(frame)) {
                        prop_assert!(point.distance(near) >= MIN_DISTANCE);
                    }
                }
            }
        }
    }
}
