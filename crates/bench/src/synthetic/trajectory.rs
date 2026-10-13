use std::iter;
use std::num::NonZeroU64;

use types::frame::FrameIndex;

use super::{Class, Point, Target};
use crate::random::Xoshiro256;

#[cfg(test)]
mod tests;

pub(crate) const SACCADE_PERIOD: NonZeroU64 = NonZeroU64::MIN.saturating_add(2);
pub(crate) const FLICKER: (f64, f64) = (0.6, 1.0);
const LENGTH_SAMPLES: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Flight {
    pub controls: [Point; 3],
    pub speed: f64,
    pub acceleration: f64,
}

pub(crate) fn saccade_progress(frames: usize, phase: u64) -> Vec<f64> {
    let period = SACCADE_PERIOD.get();
    let steps: Vec<f64> = (1..frames as u64)
        .map(|frame| {
            if (frame + phase).is_multiple_of(period) {
                2.0
            } else {
                1.0
            }
        })
        .collect();
    let total: f64 = steps.iter().sum();
    let mut reached = 0.0;
    iter::once(0.0)
        .chain(steps.into_iter().map(|step| {
            reached += step / total;
            reached
        }))
        .collect()
}

pub(crate) fn bezier([start, control, end]: [Point; 3], at: f64) -> Point {
    let (a, b, c) = ((1.0 - at).powi(2), 2.0 * (1.0 - at) * at, at.powi(2));
    Point {
        x: a * start.x + b * control.x + c * end.x,
        y: a * start.y + b * control.y + c * end.y,
    }
}

pub(crate) fn bezier_length(controls: [Point; 3]) -> f64 {
    let points: Vec<Point> = (0..=LENGTH_SAMPLES)
        .map(|sample| bezier(controls, f64::from(sample) / f64::from(LENGTH_SAMPLES)))
        .collect();
    points.windows(2).map(|pair| pair[0].distance(pair[1])).sum()
}

pub(crate) fn flickered(class: Class, frames: usize, random: &mut Xoshiro256) -> Vec<f64> {
    (0..frames)
        .map(|_| class.amplitude() * random.uniform(FLICKER.0, FLICKER.1))
        .collect()
}

pub(crate) fn passing_target(
    id: usize,
    start: FrameIndex,
    flight: Flight,
    class: Class,
    random: &mut Xoshiro256,
) -> Target {
    let frames = ((bezier_length(flight.controls) / flight.speed).ceil() as usize).max(2);
    let progress = saccade_progress(frames, random.below(SACCADE_PERIOD));
    let positions = progress
        .into_iter()
        .map(|at| bezier(flight.controls, at.powf(flight.acceleration)))
        .collect();
    let amplitudes = flickered(class, frames, random);
    Target {
        id,
        start,
        positions,
        amplitudes,
        class,
    }
}
