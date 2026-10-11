use std::mem;

use types::detection::Detection;
use types::frame::FrameIndex;

#[cfg(test)]
mod tests;

const VELOCITY_SPAN_FRAMES: usize = 3;

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
        let span_start = self
            .earlier
            .iter()
            .rev()
            .find(|point| last.frame.abs_diff(point.frame) >= VELOCITY_SPAN_FRAMES)
            .or_else(|| self.earlier.last());
        let Some(start) = span_start else {
            return (last.x, last.y);
        };
        let elapsed = frames_between(start.frame, last.frame);
        let ahead = frames_between(last.frame, frame);
        (
            last.x + (last.x - start.x) / elapsed * ahead,
            last.y + (last.y - start.y) / elapsed * ahead,
        )
    }

    pub(crate) fn into_points(mut self) -> Vec<Detection> {
        self.earlier.push(self.last);
        self.earlier
    }
}

fn frames_between(from: FrameIndex, to: FrameIndex) -> f64 {
    from.abs_diff(to) as f64
}
