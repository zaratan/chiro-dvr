use std::num::{NonZeroU32, NonZeroUsize};

use engine::config::DetectConfig;
use render::config::RenderConfig;
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Change {
    Threshold(f64),
    MinHits(NonZeroUsize),
    MaxBlobs(Option<NonZeroU32>),
    WorkWidth(NonZeroU32),
    OsdRegions(usize),
}

impl Options {
    pub fn changed(&self) -> Vec<Change> {
        let defaults = Self::default();
        let (detect, default_detect) = (&self.detect, &defaults.detect);
        [
            (detect.threshold().to_bits() != default_detect.threshold().to_bits())
                .then_some(Change::Threshold(detect.threshold())),
            (self.track.min_hits() != defaults.track.min_hits()).then_some(Change::MinHits(self.track.min_hits())),
            (self.stability.max_blobs() != defaults.stability.max_blobs())
                .then_some(Change::MaxBlobs(self.stability.max_blobs())),
            (detect.work_width() != default_detect.work_width()).then_some(Change::WorkWidth(detect.work_width())),
            (!detect.osd_regions().is_empty()).then_some(Change::OsdRegions(detect.osd_regions().len())),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn tunes_the_frames(&self) -> bool {
        self.changed()
            .iter()
            .any(|change| matches!(change, Change::WorkWidth(_) | Change::OsdRegions(_)))
    }
}
