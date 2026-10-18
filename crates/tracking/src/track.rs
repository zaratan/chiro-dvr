mod association;
mod growing;
mod rejoin;
mod twins;

use std::num::NonZeroUsize;

use types::frame::FrameIndex;
use types::size::SourceSize;
use types::span::Span;
use types::track::Track;

use crate::error::Error;
use crate::exclusion::Exclusion;
use crate::setting::Setting;
use crate::track::association::Association;
use crate::track::growing::Growing;
use crate::track::rejoin::{Hiding, rejoin};
use crate::track::twins::merge_twins;

pub use crate::track::rejoin::{RejoinConfig, RejoinPace};

#[cfg(test)]
mod tests;

const DEFAULT_MAX_GAP: NonZeroUsize = match NonZeroUsize::new(6) {
    Some(frames) => frames,
    None => NonZeroUsize::MIN,
};
const DEFAULT_MIN_HITS: NonZeroUsize = match NonZeroUsize::new(6) {
    Some(points) => points,
    None => NonZeroUsize::MIN,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackConfig {
    max_jump: f64,
    max_gap: NonZeroUsize,
    min_hits: NonZeroUsize,
    min_travel: f64,
    twin_distance: Option<f64>,
    max_median_turn: f64,
    rejoin: Option<RejoinConfig>,
}

impl Default for TrackConfig {
    fn default() -> Self {
        Self {
            max_jump: 120.0,
            max_gap: DEFAULT_MAX_GAP,
            min_hits: DEFAULT_MIN_HITS,
            min_travel: 45.0,
            twin_distance: Some(36.0),
            max_median_turn: 0.8,
            rejoin: Some(RejoinConfig::default()),
        }
    }
}

impl TrackConfig {
    pub fn new(
        max_jump: f64,
        max_gap: NonZeroUsize,
        min_hits: NonZeroUsize,
        min_travel: f64,
        twin_distance: Option<f64>,
        max_median_turn: f64,
        rejoin: Option<RejoinConfig>,
    ) -> Result<Self, Error> {
        positive(max_jump, Setting::MaxJump)?;
        if !(min_travel.is_finite() && min_travel >= 0.0) {
            return Err(Error::Setting {
                setting: Setting::MinTravel,
                value: min_travel,
            });
        }
        if let Some(distance) = twin_distance {
            positive(distance, Setting::TwinDistance)?;
        }
        positive(max_median_turn, Setting::MaxMedianTurn)?;
        if let Some(hidden) = rejoin.map(RejoinConfig::max_hidden)
            && hidden <= max_gap
        {
            return Err(Error::Setting {
                setting: Setting::RejoinHidden,
                value: hidden.get() as f64,
            });
        }
        Ok(Self {
            max_jump,
            max_gap,
            min_hits,
            min_travel,
            twin_distance,
            max_median_turn,
            rejoin,
        })
    }

    pub fn max_jump(self) -> f64 {
        self.max_jump
    }

    pub fn max_gap(self) -> NonZeroUsize {
        self.max_gap
    }

    pub fn min_hits(self) -> NonZeroUsize {
        self.min_hits
    }

    pub fn min_travel(self) -> f64 {
        self.min_travel
    }

    pub fn twin_distance(self) -> Option<f64> {
        self.twin_distance
    }

    pub fn max_median_turn(self) -> f64 {
        self.max_median_turn
    }

    pub fn rejoin(self) -> Option<RejoinConfig> {
        self.rejoin
    }

    #[must_use]
    pub fn without_rejoin(self) -> Self {
        Self { rejoin: None, ..self }
    }

    fn expired(self, track: &Growing, frame: FrameIndex) -> bool {
        frame.abs_diff(track.last().frame) > self.max_gap.get().saturating_add(1)
    }

    fn keeps(self, track: &Track) -> bool {
        track.points().len() >= self.min_hits.get()
            && track.chord() >= self.min_travel
            && track.median_turn() <= self.max_median_turn
    }
}

fn positive(value: f64, setting: Setting) -> Result<(), Error> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Error::Setting { setting, value })
    }
}

pub fn track_detections(excluded: &Exclusion, size: SourceSize, config: TrackConfig) -> Result<Vec<Track>, Error> {
    let mut active: Vec<Growing> = Vec::new();
    let mut finished: Vec<Track> = Vec::new();
    let mut association = Association::default();
    for (frame, found) in excluded.detections.iter() {
        for ended in active.extract_if(.., |track| config.expired(track, frame)) {
            finished.push(Track::new(0, ended.into_points())?);
        }
        if found.is_empty() {
            continue;
        }
        let unmatched = association.assign(&mut active, found, frame, config.max_jump);
        active.extend(
            unmatched
                .filter(|detection| detection.strength.is_strong())
                .map(Growing::new),
        );
    }
    for track in active {
        finished.push(Track::new(0, track.into_points())?);
    }
    let merged = match config.twin_distance {
        Some(distance) => merge_twins(finished, distance)?,
        None => finished,
    };
    let (kept, dropped): (Vec<Track>, Vec<Track>) = merged.into_iter().partition(|track| config.keeps(track));
    let mut kept = match config.rejoin {
        Some(rejoining) => {
            let pieces: Vec<Track> = dropped
                .into_iter()
                .filter(|track| rejoining.takes_as_piece(track, config.min_hits))
                .collect();
            let ignored: Vec<Span> = excluded.ignored_spans().collect();
            let hiding = Hiding {
                longer_than: config.max_gap.get(),
                ignored: &ignored,
                size,
            };
            rejoin(&kept, &pieces, hiding, rejoining)?
        }
        None => kept,
    };
    kept.sort_by(|a, b| {
        a.first()
            .frame
            .cmp(&b.first().frame)
            .then(a.first().x.total_cmp(&b.first().x))
    });
    Ok(kept
        .into_iter()
        .enumerate()
        .map(|(index, track)| track.with_id(index + 1))
        .collect())
}
