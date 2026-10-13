use std::num::NonZeroUsize;
use std::path::Path;

use media::decode::{ActiveThreads, Threads};
use media::frame::YuvFrame;
use types::detection::FrameDetections;
use types::frame::FrameIndex;

use crate::background::BackgroundFrames;
use crate::blur::{self, Blur};
use crate::closing::Disk;
use crate::config::{self, DetectConfig};
use crate::damage::Damage;
use crate::detector::{BATCH, Detector};
use crate::geometry::{self, Geometry};
use crate::input::{self, Input, READ_AHEAD, Reading};
use crate::luma::LumaMut;
use crate::mask::{self, Mask};

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
    #[error("cannot blur the frames of {} for the detection", path.display())]
    Blur {
        path: std::path::PathBuf,
        #[source]
        source: blur::Error,
    },
    #[error("cannot mask the OSD regions of {}", path.display())]
    Mask {
        path: std::path::PathBuf,
        #[source]
        source: mask::Error,
    },
    #[error("cannot build the background window of {}", path.display())]
    Config {
        path: std::path::PathBuf,
        #[source]
        source: config::Error,
    },
}

#[derive(Debug)]
pub struct Detected {
    reading: Reading,
    detections: FrameDetections,
    damage: Damage,
    half_window: NonZeroUsize,
    active_threads: ActiveThreads,
    detection_threads: usize,
    background: Option<YuvFrame>,
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

    pub fn detection_threads(&self) -> usize {
        self.detection_threads
    }

    pub fn background(&self) -> Option<&YuvFrame> {
        self.background.as_ref()
    }
}

pub fn detect(path: &Path, threads: Threads, config: &DetectConfig) -> Result<Detected, Error> {
    detect_input(Input::open(path, threads)?, config)
}

pub fn detect_input(input: Input, config: &DetectConfig) -> Result<Detected, Error> {
    detect_injected(input, config, |_, _| {})
}

pub fn detect_injected(
    input: Input,
    config: &DetectConfig,
    mut inject: impl FnMut(FrameIndex, LumaMut<'_>),
) -> Result<Detected, Error> {
    let video = input.video();
    let path = video.path().to_path_buf();
    let geometry = Geometry::new(video.size(), config.work_width()).map_err(|source| Error::Geometry {
        path: path.clone(),
        source,
    })?;
    let blur =
        Blur::new(config.target_sigma() / geometry.scale().factor(), geometry).map_err(|source| Error::Blur {
            path: path.clone(),
            source,
        })?;
    let disk = Disk::new(config.merge_radius() / geometry.scale().factor(), geometry);
    let mask =
        Mask::new(config.osd_regions(), geometry.width(), geometry.height(), blur.reach()).map_err(|source| {
            Error::Mask {
                path: path.clone(),
                source,
            }
        })?;
    let half_window = config
        .half_window(video.frame_rate())
        .map_err(|source| Error::Config { path, source })?;
    let width = geometry.source_width();
    let active_threads = input.active_threads();
    let mut detector = Detector::new(geometry, &blur, &disk, mask, half_window, config, BATCH);
    let detection_threads = detector.workers();
    let mut background = BackgroundFrames::new(video.announced_frames());
    let reading = input.read_ahead_by(BATCH.saturating_add(READ_AHEAD.get()), |info, frame| {
        background.offer(info.index(), frame);
        detector.push(frame.y(), |copy| inject(info.index(), LumaMut::of_frame(copy, width)));
        Ok::<(), Error>(())
    })?;
    let detections = detector.finish();
    let damage = Damage::of(&reading, half_window.get());
    let background = background.into_background(damage.spans());
    Ok(Detected {
        reading,
        detections,
        damage,
        half_window,
        active_threads,
        detection_threads,
        background,
    })
}
