mod association;
mod growing;
mod twins;

use std::fmt;
use std::num::NonZeroUsize;

use types::detection::FrameDetections;
use types::frame::FrameIndex;
use types::track::Track;

use crate::error::Error;
use crate::track::association::Association;
use crate::track::growing::Growing;
use crate::track::twins::merge_twins;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackSetting {
    MaxJump,
    MinTravel,
    TwinDistance,
    MaxMedianTurn,
}

impl fmt::Display for TrackSetting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MaxJump => "max_jump",
            Self::MinTravel => "min_travel",
            Self::TwinDistance => "twin_distance",
            Self::MaxMedianTurn => "max_median_turn",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackConfig {
    max_jump: f64,
    max_gap: NonZeroUsize,
    min_hits: NonZeroUsize,
    min_travel: f64,
    twin_distance: Option<f64>,
    max_median_turn: f64,
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
    ) -> Result<Self, Error> {
        positive(max_jump, TrackSetting::MaxJump)?;
        if !(min_travel.is_finite() && min_travel >= 0.0) {
            return Err(Error::Setting {
                setting: TrackSetting::MinTravel,
                value: min_travel,
            });
        }
        if let Some(distance) = twin_distance {
            positive(distance, TrackSetting::TwinDistance)?;
        }
        positive(max_median_turn, TrackSetting::MaxMedianTurn)?;
        Ok(Self {
            max_jump,
            max_gap,
            min_hits,
            min_travel,
            twin_distance,
            max_median_turn,
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

    fn expired(self, track: &Growing, frame: FrameIndex) -> bool {
        frame.abs_diff(track.last().frame) > self.max_gap.get().saturating_add(1)
    }

    fn keeps(self, track: &Track) -> bool {
        track.points().len() >= self.min_hits.get()
            && track.chord() >= self.min_travel
            && track.median_turn() <= self.max_median_turn
    }
}

fn positive(value: f64, setting: TrackSetting) -> Result<(), Error> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Error::Setting { setting, value })
    }
}

pub fn track_detections(detections: &FrameDetections, config: TrackConfig) -> Result<Vec<Track>, Error> {
    let mut active: Vec<Growing> = Vec::new();
    let mut finished: Vec<Track> = Vec::new();
    let mut association = Association::default();
    for (frame, found) in detections.iter() {
        for ended in active.extract_if(.., |track| config.expired(track, frame)) {
            finished.push(Track::new(0, ended.into_points())?);
        }
        if found.is_empty() {
            continue;
        }
        let unmatched = association.assign(&mut active, found, frame, config.max_jump);
        active.extend(unmatched.map(Growing::new));
    }
    for track in active {
        finished.push(Track::new(0, track.into_points())?);
    }
    let merged = match config.twin_distance {
        Some(distance) => merge_twins(finished, distance)?,
        None => finished,
    };
    let mut kept: Vec<Track> = merged.into_iter().filter(|track| config.keeps(track)).collect();
    kept.sort_by(|a, b| {
        a.first()
            .frame
            .cmp(&b.first().frame)
            .then(a.first().x.total_cmp(&b.first().x))
    });
    kept.into_iter()
        .enumerate()
        .map(|(index, track)| Ok(Track::new(index + 1, track.points().to_vec())?))
        .collect()
}
