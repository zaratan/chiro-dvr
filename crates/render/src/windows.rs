use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::span::{Run, Span, merged_by};
use types::track::Track;

use crate::config::RenderConfig;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub track: usize,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    span: Span,
    members: Vec<Window>,
}

impl Group {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn members(&self) -> &[Window] {
        &self.members
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
    let mut by_track = windows.to_vec();
    by_track.sort_by_key(|window| window.track);
    merged_by(by_track, |window| window.span)
        .into_iter()
        .map(|Run { span, members }| Group { span, members })
        .collect()
}
