use std::mem;

use types::detection::Detection;
use types::frame::FrameIndex;

#[cfg(test)]
mod tests;

const VELOCITY_SPAN_FRAMES: usize = 3;
const TAIL_POINTS: usize = 5;
const SLUMP_SHARE: f64 = 0.4;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Growing {
    earlier: Vec<Detection>,
    last: Detection,
}

impl Growing {
    pub(crate) fn new(first: Detection) -> Self {
        Self {
            earlier: Vec::new(),
            last: first,
        }
    }

    pub(crate) fn push(&mut self, next: Detection) {
        let previous = mem::replace(&mut self.last, next);
        self.earlier.push(previous);
    }

    pub(crate) fn last(&self) -> &Detection {
        &self.last
    }

    pub(crate) fn has_velocity(&self) -> bool {
        !self.earlier.is_empty()
    }

    pub(crate) fn predicted(&self, frame: FrameIndex) -> (f64, f64) {
        let last = &self.last;
        let Some((x, y)) = self.velocity() else {
            return (last.x, last.y);
        };
        let ahead = frames_between(last.frame, frame);
        (last.x + x * ahead, last.y + y * ahead)
    }

    pub(crate) fn predicted_travel(&self, frame: FrameIndex) -> Option<f64> {
        let (x, y) = self.velocity()?;
        let current = x.hypot(y);
        let tail = self.tail_speed()?;
        let trusted = if current < SLUMP_SHARE * tail { tail } else { current };
        Some(trusted * frames_between(self.last.frame, frame))
    }

    pub(crate) fn speed_measured_from(&self) -> Option<&Detection> {
        let last = &self.last;
        self.earlier
            .iter()
            .rev()
            .find(|point| last.frame.abs_diff(point.frame) >= VELOCITY_SPAN_FRAMES)
            .or_else(|| self.earlier.last())
    }

    fn velocity(&self) -> Option<(f64, f64)> {
        let (last, start) = (&self.last, self.speed_measured_from()?);
        let elapsed = frames_between(start.frame, last.frame);
        Some(((last.x - start.x) / elapsed, (last.y - start.y) / elapsed))
    }

    fn tail_speed(&self) -> Option<f64> {
        let last = &self.last;
        let start = self.earlier.get(self.earlier.len().saturating_sub(TAIL_POINTS - 1))?;
        Some((last.x - start.x).hypot(last.y - start.y) / frames_between(start.frame, last.frame))
    }

    pub(crate) fn into_points(mut self) -> Vec<Detection> {
        self.earlier.push(self.last);
        self.earlier
    }
}

fn frames_between(from: FrameIndex, to: FrameIndex) -> f64 {
    from.abs_diff(to) as f64
}
