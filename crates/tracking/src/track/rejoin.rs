use std::num::NonZeroUsize;

use types::detection::Detection;
use types::size::SourceSize;
use types::span::Span;
use types::track::Track;

use crate::error::Error;
use crate::setting::Setting;
use crate::track::positive;

#[cfg(test)]
mod tests;

const VELOCITY_POINTS: usize = 5;
const EDGE_MARGIN: f64 = 30.0;
const DEFAULT_MAX_HIDDEN: NonZeroUsize = match NonZeroUsize::new(19) {
    Some(frames) => frames,
    None => NonZeroUsize::MIN,
};
const DEFAULT_MAX_DISTANCE: f64 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RejoinConfig {
    max_hidden: NonZeroUsize,
    max_distance: f64,
}

impl Default for RejoinConfig {
    fn default() -> Self {
        Self {
            max_hidden: DEFAULT_MAX_HIDDEN,
            max_distance: DEFAULT_MAX_DISTANCE,
        }
    }
}

impl RejoinConfig {
    pub fn new(max_hidden: NonZeroUsize, max_distance: f64) -> Result<Self, Error> {
        positive(max_distance, Setting::RejoinDistance)?;
        Ok(Self {
            max_hidden,
            max_distance,
        })
    }

    pub fn max_hidden(self) -> NonZeroUsize {
        self.max_hidden
    }

    pub fn max_distance(self) -> f64 {
        self.max_distance
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Hiding<'a> {
    pub(crate) longer_than: usize,
    pub(crate) ignored: &'a [Span],
    pub(crate) size: SourceSize,
}

impl Hiding<'_> {
    fn miss(self, ended: &Track, started: &Track, config: RejoinConfig) -> Option<f64> {
        let (end, start) = (ended.last(), started.first());
        let hidden = start.frame.get().checked_sub(end.frame.get())?.checked_sub(1)?;
        if hidden <= self.longer_than
            || hidden > config.max_hidden.get()
            || self.at_edge(end)
            || self.at_edge(start)
            || self
                .ignored
                .iter()
                .any(|span| span.first() < start.frame && end.frame < span.last())
        {
            return None;
        }
        let tail = &ended.points()[ended.points().len().saturating_sub(VELOCITY_POINTS)..];
        let head = &started.points()[..started.points().len().min(VELOCITY_POINTS)];
        let ((tail_x, tail_y), (head_x, head_y)) = (velocity(tail), velocity(head));
        let frames = (hidden + 1) as f64;
        let meeting_x = end.x + f64::midpoint(tail_x, head_x) * frames;
        let meeting_y = end.y + f64::midpoint(tail_y, head_y) * frames;
        let miss = (start.x - meeting_x).hypot(start.y - meeting_y);
        (miss <= config.max_distance).then_some(miss)
    }

    fn at_edge(self, point: &Detection) -> bool {
        let width = f64::from(self.size.width().get());
        let height = f64::from(self.size.height().get());
        point.x.min(width - point.x).min(point.y).min(height - point.y) < EDGE_MARGIN
    }
}

pub(crate) fn rejoin(tracks: &[Track], hiding: Hiding<'_>, config: RejoinConfig) -> Result<Vec<Track>, Error> {
    let mut links: Vec<(f64, usize, usize)> = Vec::new();
    for (before, ended) in tracks.iter().enumerate() {
        for (after, started) in tracks.iter().enumerate() {
            if let Some(miss) = hiding.miss(ended, started, config) {
                links.push((miss, before, after));
            }
        }
    }
    links.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut next: Vec<Option<usize>> = vec![None; tracks.len()];
    let mut continued = vec![false; tracks.len()];
    for (_, before, after) in links {
        if next[before].is_none() && !continued[after] {
            next[before] = Some(after);
            continued[after] = true;
        }
    }
    let mut rejoined = Vec::new();
    for head in (0..tracks.len()).filter(|&index| !continued[index]) {
        let mut parts: Vec<Vec<Detection>> = Vec::new();
        let mut part = Some(head);
        while let Some(index) = part {
            parts.push(tracks[index].points().to_vec());
            part = next[index];
        }
        rejoined.push(Track::rejoined(0, parts)?);
    }
    Ok(rejoined)
}

fn velocity(points: &[Detection]) -> (f64, f64) {
    match (points.first(), points.last()) {
        (Some(first), Some(last)) if last.frame > first.frame => {
            let frames = last.frame.abs_diff(first.frame) as f64;
            ((last.x - first.x) / frames, (last.y - first.y) / frames)
        }
        _ => (0.0, 0.0),
    }
}
