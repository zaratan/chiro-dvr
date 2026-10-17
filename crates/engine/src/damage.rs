use types::frame::{FrameIndex, Pts};
use types::losses::EdgeLosses;
use types::span::Span;

use crate::input::Reading;
use crate::lost::frames_after_jumps;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Damage {
    reports: Reports,
    spans: Vec<Span>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reports {
    pub error_frames: Vec<FrameIndex>,
    pub jump_frames: Vec<FrameIndex>,
    pub edges: EdgeLosses,
}

impl Damage {
    pub fn of(reading: &Reading, margin: usize) -> Self {
        let frames = reading.frames();
        let pts: Vec<Option<Pts>> = frames.iter().map(|info| info.pts()).collect();
        let reports = Reports {
            error_frames: frames
                .iter()
                .filter(|info| !info.decode_errors().is_empty())
                .map(|info| info.index())
                .collect(),
            jump_frames: frames_after_jumps(&pts),
            edges: reading.losses().edges(),
        };
        Self::reported(reports, &reading.key_frames(), frames.len(), margin)
    }

    pub fn reported(mut reports: Reports, key_frames: &[FrameIndex], frame_count: usize, margin: usize) -> Self {
        for frames in [&mut reports.error_frames, &mut reports.jump_frames] {
            frames.sort_unstable();
            frames.dedup();
        }
        let keys: Vec<usize> = key_frames.iter().copied().map(FrameIndex::get).collect();
        let mut starts: Vec<usize> = reports
            .error_frames
            .iter()
            .chain(&reports.jump_frames)
            .map(|frame| frame.get())
            .chain(edges(reports.edges, frame_count))
            .collect();
        starts.sort_unstable();
        starts.dedup();
        Self {
            spans: spans(&starts, &keys, frame_count, margin),
            reports,
        }
    }

    pub fn reports(&self) -> &Reports {
        &self.reports
    }

    pub fn error_frames(&self) -> &[FrameIndex] {
        &self.reports.error_frames
    }

    pub fn jump_frames(&self) -> &[FrameIndex] {
        &self.reports.jump_frames
    }

    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    pub fn reported_frames(&self) -> usize {
        let mut frames: Vec<FrameIndex> = self.error_frames().iter().chain(self.jump_frames()).copied().collect();
        frames.sort_unstable();
        frames.dedup();
        frames.len()
    }
}

fn edges(losses: EdgeLosses, frame_count: usize) -> impl Iterator<Item = usize> {
    let before = (losses.before_first > 0).then_some(0);
    let after_the_end = (losses.after_last > 0).then_some(frame_count);
    before.into_iter().chain(after_the_end)
}

fn spans(starts: &[usize], keys: &[usize], frame_count: usize, margin: usize) -> Vec<Span> {
    let Some(last_frame) = frame_count.checked_sub(1) else {
        return Vec::new();
    };
    let video = Span::single(FrameIndex::new(0)).hull(Span::single(FrameIndex::new(last_frame)));
    let mut merged: Vec<Span> = Vec::new();
    for &start in starts {
        let next_key = keys.partition_point(|key| *key <= start);
        let stop = keys.get(next_key).map_or(last_frame, |key| key - 1).max(start);
        let until_key = Span::single(FrameIndex::new(start)).hull(Span::single(FrameIndex::new(stop)));
        let Some(damaged) = until_key.widened(margin, margin).intersection(video) else {
            continue;
        };
        match merged.last_mut() {
            Some(previous) if damaged.first().get() <= previous.last().get().saturating_add(1) => {
                *previous = previous.hull(damaged);
            }
            _ => merged.push(damaged),
        }
    }
    merged
}
