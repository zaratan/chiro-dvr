use std::num::NonZeroUsize;

use media::frame::YuvFrame;
use types::frame::FrameIndex;

#[cfg(test)]
mod tests;

pub(crate) const SUMMARY_FRAMES: usize = 25;

#[derive(Debug)]
pub(crate) struct SummaryFrames {
    ranks: Vec<usize>,
    kept: Vec<(FrameIndex, YuvFrame)>,
}

impl SummaryFrames {
    pub(crate) fn new(announced: NonZeroUsize) -> Self {
        let ranks = ranks(announced);
        Self {
            kept: Vec::with_capacity(ranks.len()),
            ranks,
        }
    }

    pub(crate) fn offer(&mut self, index: FrameIndex, frame: &YuvFrame) {
        if self.ranks.get(self.kept.len()) == Some(&index.get()) {
            self.kept.push((index, frame.clone()));
        }
    }

    pub(crate) fn into_frames(self) -> Vec<(FrameIndex, YuvFrame)> {
        self.kept
    }
}

pub(crate) fn ranks(announced: NonZeroUsize) -> Vec<usize> {
    let announced = announced.get();
    let count = announced.min(SUMMARY_FRAMES);
    (0..count)
        .filter_map(|index| (2 * index + 1).checked_mul(announced))
        .map(|scaled| scaled / (2 * count))
        .collect()
}
