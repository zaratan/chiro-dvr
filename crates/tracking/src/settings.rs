use std::num::{NonZeroU32, NonZeroUsize};

use serde::{Deserialize, Serialize};

use crate::stability::StabilityConfig;
use crate::track::{Reach, RejoinConfig, TrackConfig};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrackSettings {
    pub max_jump: f64,
    pub max_gap: usize,
    pub min_hits: usize,
    pub min_travel: f64,
    pub twin_distance: f64,
    pub max_median_turn: f64,
    #[serde(default)]
    pub rejoin_hidden: usize,
    #[serde(default)]
    pub rejoin_distance: f64,
    #[serde(default)]
    pub rejoin_piece_points: usize,
    #[serde(default)]
    pub rejoin_junction_pace: f64,
    #[serde(default)]
    pub rejoin_piece_pace: f64,
    #[serde(default)]
    pub reach_margin: f64,
    #[serde(default)]
    pub reach_travel: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StabilitySettings {
    pub max_blobs: u32,
    pub pad_s: f64,
}

impl From<TrackConfig> for TrackSettings {
    fn from(track: TrackConfig) -> Self {
        Self {
            max_jump: track.max_jump(),
            max_gap: track.max_gap().get(),
            min_hits: track.min_hits().get(),
            min_travel: track.min_travel(),
            twin_distance: track.twin_distance().unwrap_or(0.0),
            max_median_turn: track.max_median_turn(),
            rejoin_hidden: track.rejoin().map_or(0, |rejoin| rejoin.max_hidden().get()),
            rejoin_distance: track.rejoin().map_or(0.0, RejoinConfig::max_distance),
            rejoin_piece_points: track
                .rejoin()
                .and_then(RejoinConfig::piece_points)
                .map_or(0, NonZeroUsize::get),
            rejoin_junction_pace: track.rejoin().map_or(0.0, |rejoin| rejoin.pace().junction()),
            rejoin_piece_pace: track.rejoin().map_or(0.0, |rejoin| rejoin.pace().piece()),
            reach_margin: track.reach().map_or(0.0, Reach::margin),
            reach_travel: track.reach().map_or(0.0, Reach::travel),
        }
    }
}

impl From<StabilityConfig> for StabilitySettings {
    fn from(stability: StabilityConfig) -> Self {
        Self {
            max_blobs: stability.max_blobs().map_or(0, NonZeroU32::get),
            pad_s: stability.pad_s(),
        }
    }
}
