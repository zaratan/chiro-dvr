use std::num::NonZeroU32;

use types::detection::FrameDetections;
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::span::Span;

use crate::error::Error;
use crate::spans::coverage;

#[cfg(test)]
mod tests;

const HALF_TYPICAL_FACTOR: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StabilityConfig {
    max_blobs: Option<NonZeroU32>,
    pad_s: f64,
}

impl Default for StabilityConfig {
    fn default() -> Self {
        Self {
            max_blobs: NonZeroU32::new(20),
            pad_s: 1.0,
        }
    }
}

impl StabilityConfig {
    pub fn new(max_blobs: Option<NonZeroU32>, pad_s: f64) -> Result<Self, Error> {
        if !(pad_s.is_finite() && pad_s >= 0.0) {
            return Err(Error::Padding(pad_s));
        }
        Ok(Self { max_blobs, pad_s })
    }

    pub fn max_blobs(self) -> Option<NonZeroU32> {
        self.max_blobs
    }

    pub fn pad_s(self) -> f64 {
        self.pad_s
    }

    fn pad_frames(self, rate: FrameRate, frame_count: usize) -> usize {
        let frames = (self.pad_s * rate.get().to_f64()).round();
        if frames < frame_count as f64 {
            frames as usize
        } else {
            frame_count
        }
    }
}

pub(crate) fn crowd_limit(counts: &[usize], max_blobs: NonZeroU32) -> usize {
    let mut sorted = counts.to_vec();
    sorted.sort_unstable();
    let low = sorted.get(sorted.len().saturating_sub(1) / 2).copied().unwrap_or(0);
    let high = sorted.get(sorted.len() / 2).copied().unwrap_or(0);
    max_blobs.get() as usize + HALF_TYPICAL_FACTOR * (low + high)
}

pub(crate) fn unstable_spans(
    detections: &FrameDetections,
    damaged: &[Span],
    rate: FrameRate,
    config: StabilityConfig,
) -> Result<Vec<Span>, Error> {
    let Some(max_blobs) = config.max_blobs else {
        return Ok(Vec::new());
    };
    let hidden = coverage(damaged, detections.frame_count());
    let readable: Vec<(usize, usize)> = detections
        .iter()
        .zip(hidden)
        .filter(|&(_, hidden)| !hidden)
        .map(|((frame, found), _)| (frame.get(), found.len()))
        .collect();
    let (Some(&(first_readable, _)), Some(&(last_readable, _))) = (readable.first(), readable.last()) else {
        return Ok(Vec::new());
    };
    let counts: Vec<usize> = readable.iter().map(|&(_, count)| count).collect();
    let limit = crowd_limit(&counts, max_blobs);
    let pad = config.pad_frames(rate, detections.frame_count());
    let mut bounds: Vec<(usize, usize)> = Vec::new();
    for (frame, _) in readable.into_iter().filter(|&(_, count)| count >= limit) {
        let first = frame.saturating_sub(pad).max(first_readable);
        let last = (frame + pad).min(last_readable);
        match bounds.last_mut() {
            Some(previous) if first <= previous.1 + pad + 1 => previous.1 = last,
            _ => bounds.push((first, last)),
        }
    }
    bounds
        .into_iter()
        .map(|(first, last)| Ok(Span::new(FrameIndex::new(first), FrameIndex::new(last))?))
        .collect()
}
