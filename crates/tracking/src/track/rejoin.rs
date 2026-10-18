use std::num::NonZeroUsize;

use types::detection::Detection;
use types::size::SourceSize;
use types::span::Span;
use types::track::{Stretch, Track};

use crate::error::Error;
use crate::setting::Setting;
use crate::track::edge::at_edge;
use crate::track::positive;

mod pace;

#[cfg(test)]
mod tests;

pub use crate::track::rejoin::pace::RejoinPace;

const VELOCITY_POINTS: usize = 5;
const DEFAULT_MAX_HIDDEN: NonZeroUsize = match NonZeroUsize::new(19) {
    Some(frames) => frames,
    None => NonZeroUsize::MIN,
};
const DEFAULT_MAX_DISTANCE: f64 = 60.0;
const DEFAULT_PIECE_POINTS: NonZeroUsize = match NonZeroUsize::new(2) {
    Some(points) => points,
    None => NonZeroUsize::MIN,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    Kept,
    Piece,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RejoinConfig {
    max_hidden: NonZeroUsize,
    max_distance: f64,
    piece_points: Option<NonZeroUsize>,
    pace: RejoinPace,
}

impl Default for RejoinConfig {
    fn default() -> Self {
        Self {
            max_hidden: DEFAULT_MAX_HIDDEN,
            max_distance: DEFAULT_MAX_DISTANCE,
            piece_points: Some(DEFAULT_PIECE_POINTS),
            pace: RejoinPace::default(),
        }
    }
}

impl RejoinConfig {
    pub fn new(max_hidden: NonZeroUsize, max_distance: f64) -> Result<Self, Error> {
        positive(max_distance, Setting::RejoinDistance)?;
        Ok(Self {
            max_hidden,
            max_distance,
            ..Self::default()
        })
    }

    #[must_use]
    pub fn with_pieces(self, piece_points: Option<NonZeroUsize>, pace: RejoinPace) -> Self {
        Self {
            piece_points,
            pace,
            ..self
        }
    }

    pub fn piece_points(self) -> Option<NonZeroUsize> {
        self.piece_points
    }

    pub fn pace(self) -> RejoinPace {
        self.pace
    }

    pub(crate) fn takes_as_piece(self, track: &Track, min_hits: NonZeroUsize) -> bool {
        self.piece_points
            .is_some_and(|points| (points.get()..min_hits.get()).contains(&track.points().len()))
    }

    fn links(self, ended: (&Track, Role), started: (&Track, Role)) -> bool {
        !(ended.1 == Role::Piece && started.1 == Role::Piece) && self.pace.keeps(ended, started)
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
            || at_edge(end, self.size)
            || at_edge(start, self.size)
            || self
                .ignored
                .iter()
                .any(|span| span.first() < start.frame && end.frame < span.last())
        {
            return None;
        }
        let ((tail_x, tail_y), (head_x, head_y)) = (velocity(tail(ended)), velocity(head(started)));
        let frames = (hidden + 1) as f64;
        let meeting_x = end.x + f64::midpoint(tail_x, head_x) * frames;
        let meeting_y = end.y + f64::midpoint(tail_y, head_y) * frames;
        let miss = (start.x - meeting_x).hypot(start.y - meeting_y);
        (miss <= config.max_distance).then_some(miss)
    }
}

pub(crate) fn rejoin(
    tracks: &[Track],
    pieces: &[Track],
    hiding: Hiding<'_>,
    config: RejoinConfig,
) -> Result<Vec<Track>, Error> {
    let all: Vec<(&Track, Role)> = tracks
        .iter()
        .map(|track| (track, Role::Kept))
        .chain(pieces.iter().map(|piece| (piece, Role::Piece)))
        .collect();
    let mut links: Vec<(bool, f64, usize, usize)> = Vec::new();
    for (before, &ended) in all.iter().enumerate() {
        for (after, &started) in all.iter().enumerate() {
            if let Some(miss) = hiding.miss(ended.0, started.0, config)
                && config.links(ended, started)
            {
                let through_piece = ended.1 == Role::Piece || started.1 == Role::Piece;
                links.push((through_piece, miss, before, after));
            }
        }
    }
    links.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut next: Vec<Option<usize>> = vec![None; all.len()];
    let mut continued = vec![false; all.len()];
    for (_, _, before, after) in links {
        if next[before].is_none() && !continued[after] {
            next[before] = Some(after);
            continued[after] = true;
        }
    }
    let mut rejoined = Vec::new();
    for head in (0..all.len()).filter(|&index| !continued[index]) {
        let mut stretches: Vec<Stretch> = Vec::new();
        let mut link = Some(head);
        while let Some(index) = link {
            let (track, role) = all[index];
            stretches.push(match role {
                Role::Kept => Stretch::Part(track.points().to_vec()),
                Role::Piece => Stretch::Piece(track.points().to_vec()),
            });
            link = next[index];
        }
        if stretches.iter().any(|stretch| matches!(stretch, Stretch::Part(_))) {
            rejoined.push(Track::rejoined(0, stretches)?);
        }
    }
    Ok(rejoined)
}

pub(crate) fn tail(track: &Track) -> &[Detection] {
    &track.points()[track.points().len().saturating_sub(VELOCITY_POINTS)..]
}

pub(crate) fn head(track: &Track) -> &[Detection] {
    &track.points()[..track.points().len().min(VELOCITY_POINTS)]
}

pub(crate) fn velocity(points: &[Detection]) -> (f64, f64) {
    match (points.first(), points.last()) {
        (Some(first), Some(last)) if last.frame > first.frame => {
            let frames = last.frame.abs_diff(first.frame) as f64;
            ((last.x - first.x) / frames, (last.y - first.y) / frames)
        }
        _ => (0.0, 0.0),
    }
}
