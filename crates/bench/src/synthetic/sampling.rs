use std::num::{NonZeroU64, NonZeroUsize};

use types::frame::FrameIndex;
use types::track::Track;

use super::hunting::{hunting_target, side_point};
use super::trajectory::{Flight, passing_target};
use super::{Class, Motion, Point, Scene, Target};
use crate::random::{SplitMix64, Xoshiro256};

#[cfg(test)]
mod tests;

const SPEED: (f64, f64) = (15.0, 60.0);
const ACCELERATION: (f64, f64) = (1.0, 1.8);
pub(crate) const MAX_ATTEMPTS: usize = 500;
const SIDES: NonZeroU64 = NonZeroU64::MIN.saturating_add(3);
const OTHER_SIDES: NonZeroU64 = NonZeroU64::MIN.saturating_add(2);

#[derive(Debug, Clone, PartialEq)]
pub struct Sampling {
    pub seed: u64,
    pub classes: Vec<Class>,
    pub per_class: NonZeroUsize,
    pub min_distance: f64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Occupancy(Vec<Vec<Point>>);

impl Occupancy {
    pub(crate) fn of_tracks(tracks: &[Track]) -> Self {
        let mut occupancy = Self::default();
        for point in tracks.iter().flat_map(Track::points) {
            occupancy.add(point.frame, Point { x: point.x, y: point.y });
        }
        occupancy
    }

    fn add(&mut self, frame: FrameIndex, point: Point) {
        let index = frame.get();
        if self.0.len() <= index {
            self.0.resize_with(index + 1, Vec::new);
        }
        self.0[index].push(point);
    }

    fn at(&self, frame: usize) -> &[Point] {
        self.0.get(frame).map_or(&[], Vec::as_slice)
    }

    fn crowds(&self, target: &Target, min_distance: f64) -> bool {
        target.positions().iter().enumerate().any(|(offset, &point)| {
            self.at(target.start().get() + offset)
                .iter()
                .any(|&other| point.distance(other) < min_distance)
        })
    }

    fn occupy(&mut self, target: &Target) {
        for (offset, &point) in target.positions().iter().enumerate() {
            self.add(FrameIndex::new(target.start().get() + offset), point);
        }
    }
}

fn passing_shape(id: usize, class: Class, scene: Scene, random: &mut Xoshiro256) -> Target {
    let margin = class.margin();
    let first = random.below(SIDES);
    let second = (first + 1 + random.below(OTHER_SIDES)) % SIDES.get();
    let start = side_point(random, first, scene, margin);
    let end = side_point(random, second, scene, margin);
    let control = Point {
        x: random.uniform(0.0, scene.width()),
        y: random.uniform(0.0, scene.height()),
    };
    let flight = Flight {
        controls: [start, control, end],
        speed: random.uniform(SPEED.0, SPEED.1),
        acceleration: random.uniform(ACCELERATION.0, ACCELERATION.1),
    };
    passing_target(id, FrameIndex::new(0), flight, class, random)
}

pub(crate) fn shape(id: usize, class: Class, scene: Scene, random: &mut Xoshiro256) -> Target {
    match class.motion() {
        Motion::Pass => passing_shape(id, class, scene, random),
        Motion::Hunt | Motion::Circle => hunting_target(id, FrameIndex::new(0), class, scene, random),
    }
}

pub(crate) fn random_targets(sampling: &Sampling, scene: Scene, frames: usize, occupied: &Occupancy) -> Vec<Target> {
    let mut seeds = SplitMix64::new(sampling.seed);
    let mut occupancy = occupied.clone();
    let mut targets = Vec::new();
    let classes = sampling
        .classes
        .iter()
        .flat_map(|&class| std::iter::repeat_n(class, sampling.per_class.get()));
    for (id, class) in classes.enumerate() {
        let mut random = Xoshiro256::seeded(seeds.next_u64());
        if let Some(target) = placed(id, class, scene, frames, &occupancy, sampling.min_distance, &mut random) {
            occupancy.occupy(&target);
            targets.push(target);
        }
    }
    targets
}

fn placed(
    id: usize,
    class: Class,
    scene: Scene,
    frames: usize,
    occupancy: &Occupancy,
    min_distance: f64,
    random: &mut Xoshiro256,
) -> Option<Target> {
    (0..MAX_ATTEMPTS).find_map(|_| {
        let shape = shape(id, class, scene, random);
        let room = NonZeroU64::new(frames.checked_sub(shape.positions().len())? as u64)?;
        let candidate = shape.starting_at(FrameIndex::new(usize::try_from(random.below(room)).ok()?));
        (!occupancy.crowds(&candidate, min_distance)).then_some(candidate)
    })
}
