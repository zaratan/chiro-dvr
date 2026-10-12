use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::span::Span;
use types::track::Track;

use crate::config::RenderConfig;

#[cfg(test)]
mod tests;

pub const HARDWARE_LANES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub track: usize,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Member {
    pub window: Window,
    pub lane: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    span: Span,
    members: Vec<Member>,
    lanes: usize,
    hardware_ahead: Vec<usize>,
}

impl Group {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn members(&self) -> &[Member] {
        &self.members
    }

    pub fn lanes(&self) -> usize {
        self.lanes
    }

    pub fn hardware_lanes(&self) -> usize {
        self.hardware_lanes_from(self.span.first())
    }

    pub fn hardware_lanes_from(&self, frame: FrameIndex) -> usize {
        frame
            .get()
            .checked_sub(self.span.first().get())
            .and_then(|offset| self.hardware_ahead.get(offset))
            .copied()
            .unwrap_or(0)
    }

    fn with_hardware_ahead(mut self) -> Self {
        let first = self.span.first().get();
        let mut opened = vec![0_i64; self.span.frame_count() + 1];
        for member in self.members.iter().filter(|member| member.lane < HARDWARE_LANES) {
            opened[member.window.span.first().get() - first] += 1;
            opened[member.window.span.last().get() + 1 - first] -= 1;
        }
        let mut open = 0;
        let mut counts: Vec<usize> = opened
            .iter()
            .take(self.span.frame_count())
            .map(|change| {
                open += change;
                usize::try_from(open).unwrap_or(0)
            })
            .collect();
        for index in (0..counts.len().saturating_sub(1)).rev() {
            counts[index] = counts[index].max(counts[index + 1]);
        }
        self.hardware_ahead = counts;
        self
    }
}

pub fn clip_windows(tracks: &[Track], frame_count: usize, config: RenderConfig, rate: FrameRate) -> Vec<Window> {
    let Some(last_frame) = frame_count.checked_sub(1) else {
        return Vec::new();
    };
    let video = Span::single(FrameIndex::new(0)).hull(Span::single(FrameIndex::new(last_frame)));
    let margin = config.margin_frames(rate);
    tracks
        .iter()
        .enumerate()
        .filter_map(|(track, points)| {
            let seen = Span::single(points.first().frame).hull(Span::single(points.last().frame));
            let span = seen.widened(margin, margin).intersection(video)?;
            Some(Window { track, span })
        })
        .collect()
}

pub fn groups(windows: &[Window]) -> Vec<Group> {
    let mut sorted = windows.to_vec();
    sorted.sort_by_key(|window| (window.span.first(), window.track));
    let mut groups: Vec<Group> = Vec::new();
    let mut lane_ends: Vec<FrameIndex> = Vec::new();
    for window in sorted {
        let first = window.span.first();
        match groups.last_mut() {
            Some(group) if first.get() <= group.span.last().get().saturating_add(1) => {
                group.span = group.span.hull(window.span);
            }
            _ => {
                lane_ends.clear();
                groups.push(Group {
                    span: window.span,
                    members: Vec::new(),
                    lanes: 0,
                    hardware_ahead: Vec::new(),
                });
            }
        }
        let lane = lane_ends.iter().position(|end| *end < first).unwrap_or(lane_ends.len());
        if lane == lane_ends.len() {
            lane_ends.push(window.span.last());
        } else {
            lane_ends[lane] = window.span.last();
        }
        if let Some(group) = groups.last_mut() {
            group.members.push(Member { window, lane });
            group.lanes = lane_ends.len();
        }
    }
    groups.into_iter().map(Group::with_hardware_ahead).collect()
}
