use std::num::NonZeroUsize;
use std::path::PathBuf;

use engine::settings::DetectSettings;
use tracking::settings::{StabilitySettings, TrackSettings};
use types::detection::FrameDetections;
use types::rate::FrameRate;
use types::size::SourceSize;
use types::span::Span;

use crate::synthetic::Scene;
use crate::synthetic::injection::Injected;
use crate::synthetic::sampling::Sampling;

#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    pub path: PathBuf,
    pub size: SourceSize,
    pub rate: FrameRate,
    pub damaged: Vec<Span>,
    pub reported_frames: usize,
    pub half_window: NonZeroUsize,
}

impl Video {
    pub fn scene(&self) -> Scene {
        Scene::new(self.size, self.rate)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Provenance {
    Drawn {
        sampling: Sampling,
        track: TrackSettings,
        stability: StabilitySettings,
    },
    Replayed(PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub video: Video,
    pub detect: DetectSettings,
    pub reference: FrameDetections,
    pub injected: FrameDetections,
    pub targets: Vec<Injected>,
    pub provenance: Provenance,
}
