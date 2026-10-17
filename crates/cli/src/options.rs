use std::num::{NonZeroU32, NonZeroUsize};

use engine::config::DetectConfig;
use render::config::{RenderConfig, Zoom};
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

use crate::settings::{self, DETECT, Leaf, RENDER, STABILITY, TRACK};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Options {
    pub detect: DetectConfig,
    pub track: TrackConfig,
    pub stability: StabilityConfig,
    pub render: RenderConfig,
    pub annotated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    Zoom(Zoom),
    Annotated,
    MaxTracks(usize),
    ClipMargin(f64),
    OsdRegions(usize),
    Threshold(f64),
    MinHits(NonZeroUsize),
    MaxBlobs(Option<NonZeroU32>),
    WorkWidth(NonZeroU32),
    Unnamed(String),
}

impl Change {
    const fn order(&self) -> usize {
        match self {
            Self::Zoom(_) => 0,
            Self::Annotated => 1,
            Self::MaxTracks(_) => 2,
            Self::ClipMargin(_) => 3,
            Self::OsdRegions(_) => 4,
            Self::Threshold(_) => 5,
            Self::MinHits(_) => 6,
            Self::MaxBlobs(_) => 7,
            Self::WorkWidth(_) => 8,
            Self::Unnamed(_) => 9,
        }
    }
}

impl Options {
    pub fn changed(&self) -> Vec<Change> {
        let mut changes: Vec<Change> = settings::differing(self, &Self::default())
            .iter()
            .map(|leaf| self.change(leaf))
            .collect();
        changes.sort_by_key(Change::order);
        changes
    }

    fn change(&self, leaf: &Leaf) -> Change {
        match (leaf.section.as_str(), leaf.field.as_str()) {
            (RENDER, "zoom") => Change::Zoom(self.render.zoom().choice()),
            (RENDER, "annotated") => Change::Annotated,
            (RENDER, "max_tracks") => Change::MaxTracks(self.render.max_tracks()),
            (RENDER, "clip_margin_s") => Change::ClipMargin(self.render.clip_margin_s()),
            (DETECT, "osd_regions") => Change::OsdRegions(self.detect.osd_regions().len()),
            (DETECT, "threshold") => Change::Threshold(self.detect.threshold()),
            (TRACK, "min_hits") => Change::MinHits(self.track.min_hits()),
            (STABILITY, "max_blobs") => Change::MaxBlobs(self.stability.max_blobs()),
            (DETECT, "work_width") => Change::WorkWidth(self.detect.work_width()),
            (section, field) => Change::Unnamed(format!("{section}.{field}")),
        }
    }

    pub fn tunes_the_frames(&self) -> bool {
        self.changed()
            .iter()
            .any(|change| matches!(change, Change::WorkWidth(_) | Change::OsdRegions(_)))
    }
}
