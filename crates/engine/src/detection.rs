use std::num::NonZeroUsize;
use std::path::Path;

use media::decode::{ActiveThreads, Threads};
use media::frame::YuvFrame;
use types::detection::{self, FrameDetections};
use types::frame::FrameIndex;

use crate::config::{self, DetectConfig};
use crate::damage::{self, Damage};
use crate::detector::{BATCH, Detector};
use crate::geometry::{self, Geometry};
use crate::input::{self, Input, READ_AHEAD, Reading};
use crate::summary_frames::SummaryFrames;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Input(#[from] input::Error),
    #[error("cannot reduce the frames of {} for the detection", path.display())]
    Geometry {
        path: std::path::PathBuf,
        #[source]
        source: geometry::Error,
    },
    #[error("cannot build the background window of {}", path.display())]
    Config {
        path: std::path::PathBuf,
        #[source]
        source: config::Error,
    },
    #[error(transparent)]
    Detections(#[from] detection::Error),
    #[error(transparent)]
    Damage(#[from] damage::Error),
}

#[derive(Debug)]
pub struct Detected {
    reading: Reading,
    detections: FrameDetections,
    damage: Damage,
    half_window: NonZeroUsize,
    active_threads: ActiveThreads,
    summary_frames: Vec<(FrameIndex, YuvFrame)>,
}

impl Detected {
    pub fn reading(&self) -> &Reading {
        &self.reading
    }

    pub fn detections(&self) -> &FrameDetections {
        &self.detections
    }

    pub fn damage(&self) -> &Damage {
        &self.damage
    }

    pub fn half_window(&self) -> NonZeroUsize {
        self.half_window
    }

    pub fn active_threads(&self) -> ActiveThreads {
        self.active_threads
    }

    pub fn summary_frames(&self) -> &[(FrameIndex, YuvFrame)] {
        &self.summary_frames
    }
}

pub fn detect(path: &Path, threads: Threads, config: &DetectConfig) -> Result<Detected, Error> {
    detect_input(Input::open(path, threads)?, config)
}

pub fn detect_input(input: Input, config: &DetectConfig) -> Result<Detected, Error> {
    let video = input.video();
    let path = video.path().to_path_buf();
    let geometry = Geometry::new(video.size(), config.work_width()).map_err(|source| Error::Geometry {
        path: path.clone(),
        source,
    })?;
    let half_window = config
        .half_window(video.frame_rate())
        .map_err(|source| Error::Config { path, source })?;
    let active_threads = input.active_threads();
    let mut detector = Detector::new(geometry, half_window, config, BATCH);
    let mut summary = SummaryFrames::new(video.announced_frames());
    let reading = input.read_ahead_by(BATCH.saturating_add(READ_AHEAD.get()), |info, frame| {
        summary.offer(info.index(), frame);
        detector.push(frame.y());
        Ok::<(), Error>(())
    })?;
    let detections = FrameDetections::new(detector.finish())?;
    let damage = Damage::of(&reading, half_window.get())?;
    Ok(Detected {
        reading,
        detections,
        damage,
        half_window,
        active_threads,
        summary_frames: summary.into_frames(),
    })
}
