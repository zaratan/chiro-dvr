use engine::gray::gray;
use engine::luma::LumaMut;
use types::frame::FrameIndex;

use super::{Point, Target};

#[cfg(test)]
mod tests;

const NEAREST_LUMA: [u8; 256] = nearest_lumas();

const fn nearest_lumas() -> [u8; 256] {
    let mut nearest = [0; 256];
    let mut ties_seen = 0;
    let mut aimed = 0;
    while aimed < 256 {
        let darker = darkest_closest_luma(aimed);
        let tied = darker < 255 && distance(darker, aimed) == 1 && distance(darker + 1, aimed) == 1;
        let chosen = if tied && ties_seen % 2 == 1 { darker + 1 } else { darker };
        nearest[aimed] = chosen as u8;
        if tied {
            ties_seen += 1;
        }
        aimed += 1;
    }
    nearest
}

const fn darkest_closest_luma(aimed: usize) -> usize {
    let mut best = 0;
    let mut luma = 1;
    while luma < 256 {
        if distance(luma, aimed) < distance(best, aimed) {
            best = luma;
        }
        luma += 1;
    }
    best
}

const fn distance(luma: usize, aimed: usize) -> usize {
    (gray(luma as u8) as usize).abs_diff(aimed)
}

pub(crate) fn nearest_luma(aimed: u8) -> u8 {
    NEAREST_LUMA[usize::from(aimed)]
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Window {
    pub left: usize,
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
}

pub fn window(center: Point, sigma: f64, width: usize, height: usize) -> Option<Window> {
    let reach = (3.0 * sigma).ceil();
    let (x, y) = (center.x.floor(), center.y.floor());
    let bounded = |middle: f64, length: usize| {
        let first = (middle - reach).max(0.0);
        let end = (middle + reach + 1.0).min(length as f64);
        (first < end).then_some((first as usize, end as usize))
    };
    let (left, right) = bounded(x, width)?;
    let (top, bottom) = bounded(y, height)?;
    Some(Window {
        left,
        top,
        right,
        bottom,
    })
}

pub fn gray_change(amplitude: f64, sigma: f64, center: Point, x: usize, y: usize) -> f64 {
    let squared = (x as f64 - center.x).powi(2) + (y as f64 - center.y).powi(2);
    (amplitude * (-squared / (2.0 * sigma * sigma)).exp()).floor()
}

pub fn inject(plane: &mut LumaMut<'_>, target: &Target, frame: FrameIndex) -> Option<f64> {
    let center = target.position(frame)?;
    let amplitude = target.amplitude_at(frame)?;
    let sigma = target.class().sigma();
    let area = window(center, sigma, plane.width(), plane.height())?;
    let mut darkest = f64::INFINITY;
    let mut brightest = f64::NEG_INFINITY;
    for (y, row) in plane.rows_mut().enumerate().take(area.bottom).skip(area.top) {
        for (x, value) in row.iter_mut().enumerate().take(area.right).skip(area.left) {
            let before = f64::from(gray(*value));
            let aimed = (before + gray_change(amplitude, sigma, center, x, y)).clamp(0.0, 255.0) as u8;
            *value = nearest_luma(aimed);
            let change = f64::from(gray(*value)) - before;
            darkest = darkest.min(change);
            brightest = brightest.max(change);
        }
    }
    Some(if target.class().amplitude() < 0.0 {
        darkest
    } else {
        brightest
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observation {
    pub frame: FrameIndex,
    pub at: Point,
    pub effective: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Injected {
    pub target: Target,
    pub observations: Vec<Observation>,
}

#[derive(Debug)]
pub struct Injector {
    injected: Vec<Injected>,
    active: Vec<Vec<usize>>,
}

impl Injector {
    pub fn new(targets: Vec<Target>) -> Self {
        let mut active: Vec<Vec<usize>> = Vec::new();
        for (index, target) in targets.iter().enumerate() {
            let end = target.end().get();
            if active.len() <= end {
                active.resize_with(end + 1, Vec::new);
            }
            for frame in &mut active[target.start().get()..=end] {
                frame.push(index);
            }
        }
        let injected = targets
            .into_iter()
            .map(|target| Injected {
                target,
                observations: Vec::new(),
            })
            .collect();
        Self { injected, active }
    }

    pub fn inject(&mut self, frame: FrameIndex, mut plane: LumaMut<'_>) {
        let Some(here) = self.active.get(frame.get()) else {
            return;
        };
        for &index in here {
            let Injected { target, observations } = &mut self.injected[index];
            if let Some(effective) = inject(&mut plane, target, frame)
                && let Some(at) = target.position(frame)
            {
                observations.push(Observation { frame, at, effective });
            }
        }
    }

    pub fn finish(self) -> Vec<Injected> {
        self.injected
    }
}
