use std::num::NonZeroUsize;

use media::frame::YuvFrame;
use types::frame::FrameIndex;
use types::span::Span;

#[cfg(test)]
mod tests;

pub(crate) const BACKGROUND_FRAMES: usize = 25;

#[derive(Debug)]
pub(crate) struct BackgroundFrames {
    ranks: Vec<usize>,
    kept: Vec<(FrameIndex, YuvFrame)>,
}

impl BackgroundFrames {
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

    pub(crate) fn into_background(self, damaged: &[Span]) -> Option<YuvFrame> {
        let readable: Vec<&YuvFrame> = self
            .kept
            .iter()
            .filter(|(index, _)| !damaged.iter().any(|span| span.contains(*index)))
            .map(|(_, frame)| frame)
            .collect();
        if readable.is_empty() {
            median(&self.kept.iter().map(|(_, frame)| frame).collect::<Vec<_>>())
        } else {
            median(&readable)
        }
    }
}

pub(crate) fn ranks(announced: NonZeroUsize) -> Vec<usize> {
    let announced = announced.get();
    let count = announced.min(BACKGROUND_FRAMES);
    (0..count)
        .filter_map(|index| (2 * index + 1).checked_mul(announced))
        .map(|scaled| scaled / (2 * count))
        .collect()
}

pub(crate) fn median(frames: &[&YuvFrame]) -> Option<YuvFrame> {
    let mut background = (*frames.first()?).clone();
    let mut values = Vec::with_capacity(frames.len());
    let luma: Vec<&[u8]> = frames.iter().map(|frame| frame.y()).collect();
    plane_median(background.y_mut(), &luma, &mut values);
    let blue: Vec<&[u8]> = frames.iter().map(|frame| frame.u()).collect();
    plane_median(background.u_mut(), &blue, &mut values);
    let red: Vec<&[u8]> = frames.iter().map(|frame| frame.v()).collect();
    plane_median(background.v_mut(), &red, &mut values);
    Some(background)
}

fn plane_median(target: &mut [u8], sources: &[&[u8]], values: &mut Vec<u8>) {
    for (sample, level) in target.iter_mut().enumerate() {
        values.clear();
        values.extend(sources.iter().filter_map(|plane| plane.get(sample)));
        if let Some(middle) = middle(values) {
            *level = middle;
        }
    }
}

pub(crate) fn middle(values: &mut [u8]) -> Option<u8> {
    let half = values.len() / 2;
    if half >= values.len() {
        return None;
    }
    let odd = values.len() % 2 == 1;
    let (lower, upper, _) = values.select_nth_unstable(half);
    let upper = *upper;
    if odd {
        return Some(upper);
    }
    let lower = lower.iter().max().copied().unwrap_or(upper);
    Some(((u16::from(lower) + u16::from(upper)).div_ceil(2)) as u8)
}
