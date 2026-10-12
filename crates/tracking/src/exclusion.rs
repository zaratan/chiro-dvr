use types::detection::FrameDetections;
use types::rate::FrameRate;
use types::span::Span;

use crate::spans::coverage;
use crate::stability::{StabilityConfig, unstable_spans};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Exclusion {
    pub detections: FrameDetections,
    pub damaged: Vec<Span>,
    pub unstable: Vec<Span>,
    pub reported_frames: usize,
    pub ignored_frame_count: usize,
    pub damaged_frame_count: usize,
}

impl Exclusion {
    pub fn ignored_spans(&self) -> impl Iterator<Item = Span> + '_ {
        self.damaged.iter().chain(&self.unstable).copied()
    }
}

fn count(covered: &[bool]) -> usize {
    covered.iter().filter(|&&covered| covered).count()
}

pub fn exclude(
    detections: &FrameDetections,
    damaged: Vec<Span>,
    reported_frames: usize,
    rate: FrameRate,
    stability: StabilityConfig,
) -> Exclusion {
    let unstable = unstable_spans(detections, &damaged, rate, stability);
    let ignored: Vec<Span> = damaged.iter().chain(&unstable).copied().collect();
    Exclusion {
        detections: detections.without(&ignored),
        ignored_frame_count: count(&coverage(&ignored, detections.frame_count())),
        damaged_frame_count: count(&coverage(&damaged, detections.frame_count())),
        damaged,
        unstable,
        reported_frames,
    }
}
