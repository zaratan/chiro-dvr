use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::span::Span;
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
    let mut sorted = windows.to_vec();
    sorted.sort_by_key(|window| (window.span.first(), window.track));
    let mut groups: Vec<Group> = Vec::new();
    for window in sorted {
        match groups.last_mut() {
            Some(group) if window.span.first().get() <= group.span.last().get().saturating_add(1) => {
                group.span = group.span.hull(window.span);
                group.members.push(window);
            }
            _ => groups.push(Group {
                span: window.span,
                members: vec![window],
            }),
        }
    }
    groups
}
