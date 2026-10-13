use std::collections::VecDeque;
use std::f64::consts::{FRAC_PI_4, PI};
use std::num::NonZeroU64;

use types::frame::FrameIndex;

use super::trajectory::{SACCADE_PERIOD, flickered, saccade_progress};
use super::{Class, Motion, Point, Scene, Target};
use crate::random::Xoshiro256;

#[cfg(test)]
mod tests;

pub(crate) const CRUISE_SPEED: (f64, f64) = (8.0, 30.0);
const APEX_SPEED: (f64, f64) = (0.25, 0.5);
const TURN_ANGLE: (f64, f64) = (FRAC_PI_4, PI);
const APEX_TURN_RATE: (f64, f64) = (0.15, 0.4);
const MIN_TURN_FRAMES: u32 = 4;
const MAX_TURN_FRAMES: u32 = 60;
pub(crate) const MAX_DURATION_S: f64 = 4.0;
const FIRST_STRAIGHT_S: (f64, f64) = (0.1, 0.3);
const HUNT_STRAIGHT_S: (f64, f64) = (0.2, 1.0);
const CIRCLE_STRAIGHT_S: (f64, f64) = (0.1, 0.3);
pub(crate) const STRAIGHT_DRIFT: f64 = 0.05;
const ENTRY_SPREAD: f64 = FRAC_PI_4;
const SIDES: NonZeroU64 = NonZeroU64::MIN.saturating_add(3);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Step {
    pub speed: f64,
    pub turn: f64,
}

pub(crate) fn turn_profile(cruise: f64, apex_ratio: f64, apex_rate: f64, angle: f64) -> Vec<Step> {
    let mut ratios = Vec::new();
    let mut rates = Vec::new();
    for frames in MIN_TURN_FRAMES..=MAX_TURN_FRAMES {
        ratios = (0..frames)
            .map(|frame| 1.0 - (1.0 - apex_ratio) * (PI * (f64::from(frame) + 0.5) / f64::from(frames)).sin().powi(2))
            .collect();
        rates = ratios.iter().map(|ratio| apex_rate * apex_ratio / ratio).collect();
        if rates.iter().sum::<f64>() >= angle {
            break;
        }
    }
    let scale = (angle / rates.iter().sum::<f64>()).min(1.0);
    ratios
        .into_iter()
        .zip(rates)
        .map(|(ratio, rate)| Step {
            speed: cruise * ratio,
            turn: rate * scale,
        })
        .collect()
}

fn straight(random: &mut Xoshiro256, cruise: f64, seconds: f64, fps: f64) -> Vec<Step> {
    let frames = ((seconds * fps).round() as usize).max(1);
    (0..frames)
        .map(|_| Step {
            speed: cruise,
            turn: random.uniform(-STRAIGHT_DRIFT, STRAIGHT_DRIFT),
        })
        .collect()
}

struct Maneuvers<'a> {
    random: &'a mut Xoshiro256,
    motion: Motion,
    cruise: f64,
    fps: f64,
    pending: VecDeque<Step>,
}

impl<'a> Maneuvers<'a> {
    fn new(random: &'a mut Xoshiro256, motion: Motion, cruise: f64, fps: f64) -> Self {
        let seconds = random.uniform(FIRST_STRAIGHT_S.0, FIRST_STRAIGHT_S.1);
        let pending = straight(random, cruise, seconds, fps).into();
        Self {
            random,
            motion,
            cruise,
            fps,
            pending,
        }
    }

    fn turn_then_straight(&mut self) {
        let apex_ratio = self.random.uniform(APEX_SPEED.0, APEX_SPEED.1);
        let apex_rate = self.random.uniform(APEX_TURN_RATE.0, APEX_TURN_RATE.1);
        let angle = self.random.uniform(TURN_ANGLE.0, TURN_ANGLE.1);
        let sign = self.random.sign();
        self.pending.extend(
            turn_profile(self.cruise, apex_ratio, apex_rate, angle)
                .into_iter()
                .map(|step| Step {
                    turn: sign * step.turn,
                    ..step
                }),
        );
        let (shortest, longest) = match self.motion {
            Motion::Circle => CIRCLE_STRAIGHT_S,
            Motion::Pass | Motion::Hunt => HUNT_STRAIGHT_S,
        };
        let seconds = self.random.uniform(shortest, longest);
        let straight = straight(self.random, self.cruise, seconds, self.fps);
        self.pending.extend(straight);
    }
}

impl Iterator for Maneuvers<'_> {
    type Item = Step;

    fn next(&mut self) -> Option<Step> {
        if self.pending.is_empty() {
            self.turn_then_straight();
        }
        self.pending.pop_front()
    }
}

pub(crate) fn side_point(random: &mut Xoshiro256, side: u64, scene: Scene, margin: f64) -> Point {
    let along = random.uniform(0.0, 1.0);
    match side {
        0 => Point {
            x: along * scene.width(),
            y: -margin,
        },
        1 => Point {
            x: scene.width() + margin,
            y: along * scene.height(),
        },
        2 => Point {
            x: along * scene.width(),
            y: scene.height() + margin,
        },
        _ => Point {
            x: -margin,
            y: along * scene.height(),
        },
    }
}

fn entry(random: &mut Xoshiro256, scene: Scene, margin: f64) -> (Point, f64) {
    let side = random.below(SIDES);
    let start = side_point(random, side, scene, margin);
    let heading = (scene.height() / 2.0 - start.y).atan2(scene.width() / 2.0 - start.x)
        + random.uniform(-ENTRY_SPREAD, ENTRY_SPREAD);
    (start, heading)
}

pub(crate) fn hunting_path(random: &mut Xoshiro256, class: Class, scene: Scene) -> Vec<Point> {
    let margin = class.margin();
    let (mut at, mut heading) = entry(random, scene, margin);
    let mut points = vec![at];
    let mut entered = false;
    let limit = (MAX_DURATION_S * scene.fps()).round() as usize;
    let cruise = random.uniform(CRUISE_SPEED.0, CRUISE_SPEED.1);
    let fps = scene.fps();
    for step in Maneuvers::new(random, class.motion(), cruise, fps) {
        heading += step.turn;
        at = Point {
            x: at.x + step.speed * heading.cos(),
            y: at.y + step.speed * heading.sin(),
        };
        points.push(at);
        let inside = (0.0..=scene.width()).contains(&at.x) && (0.0..=scene.height()).contains(&at.y);
        entered = entered || inside;
        let near =
            (-margin..=scene.width() + margin).contains(&at.x) && (-margin..=scene.height() + margin).contains(&at.y);
        if points.len() >= limit || (entered && !near) {
            break;
        }
    }
    points
}

pub(crate) fn with_saccade(points: &[Point], phase: u64) -> Vec<Point> {
    let last = points.len().saturating_sub(1);
    saccade_progress(points.len(), phase)
        .into_iter()
        .map(|progress| interpolated(points, progress * last as f64))
        .collect()
}

fn interpolated(points: &[Point], time: f64) -> Point {
    let last = points.len() - 1;
    let before = (time.floor().max(0.0) as usize).min(last.saturating_sub(1));
    let after = (before + 1).min(last);
    let fraction = (time - before as f64).clamp(0.0, 1.0);
    let (from, to) = (points[before], points[after]);
    Point {
        x: from.x + (to.x - from.x) * fraction,
        y: from.y + (to.y - from.y) * fraction,
    }
}

pub(crate) fn hunting_target(
    id: usize,
    start: FrameIndex,
    class: Class,
    scene: Scene,
    random: &mut Xoshiro256,
) -> Target {
    let path = hunting_path(random, class, scene);
    let positions = with_saccade(&path, random.below(SACCADE_PERIOD));
    let amplitudes = flickered(class, positions.len(), random);
    Target {
        id,
        start,
        positions,
        amplitudes,
        class,
    }
}
