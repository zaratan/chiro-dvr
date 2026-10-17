use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use engine::config::DetectConfig;
use engine::detection::{self, Detected, detect, detect_injected};
use engine::input::{self, Input};
use engine::settings::DetectSettings;
use media::decode::Threads;
use tracking::exclusion::exclude;
use tracking::settings::{StabilitySettings, TrackSettings};
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};

use crate::run::{Provenance, Run, Video};
use crate::synthetic::Target;
use crate::synthetic::injection::Injector;
use crate::synthetic::sampling::{Occupancy, Sampling, random_targets};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub enum Targets {
    Drawn {
        sampling: Sampling,
        track: TrackConfig,
        stability: StabilityConfig,
    },
    Replayed {
        path: PathBuf,
        targets: Vec<Target>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Durations {
    pub reference: Duration,
    pub injected: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Collected {
    pub run: Run,
    pub durations: Durations,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the reference detection failed")]
    Reference(#[source] detection::Error),
    #[error("cannot open the video again for the injected detection")]
    Input(#[source] input::Error),
    #[error("the injected detection failed")]
    Injected(#[source] detection::Error),
    #[error("the tracking of the reference detection failed")]
    Tracking(#[source] tracking::error::Error),
    #[error("target {id} ends at frame {end}, beyond the {frames} frames of the video")]
    Beyond { id: usize, end: usize, frames: usize },
}

pub fn collect(video: &Path, threads: Threads, config: &DetectConfig, targets: Targets) -> Result<Collected, Error> {
    let started = Instant::now();
    let reference = detect(video, threads, config).map_err(Error::Reference)?;
    let reference_seconds = started.elapsed();
    let frames = reference.detections().frame_count();
    let described = describe(&reference);
    let (targets, provenance) = match targets {
        Targets::Drawn {
            sampling,
            track,
            stability,
        } => {
            let excluded = exclude(
                reference.detections(),
                described.damaged.clone(),
                described.reported_frames,
                described.rate,
                stability,
            );
            let tracks = track_detections(&excluded, described.size, track).map_err(Error::Tracking)?;
            let drawn = random_targets(&sampling, described.scene(), frames, &Occupancy::of_tracks(&tracks));
            let provenance = Provenance::Drawn {
                sampling,
                track: TrackSettings::from(track),
                stability: StabilitySettings::from(stability),
            };
            (drawn, provenance)
        }
        Targets::Replayed { path, targets } => {
            if let Some(beyond) = targets.iter().find(|target| target.end().get() >= frames) {
                return Err(Error::Beyond {
                    id: beyond.id(),
                    end: beyond.end().get(),
                    frames,
                });
            }
            (targets, Provenance::Replayed(path))
        }
    };
    let started = Instant::now();
    let mut injector = Injector::new(targets);
    let input = Input::open(video, threads).map_err(Error::Input)?;
    let injected =
        detect_injected(input, config, |frame, plane| injector.inject(frame, plane)).map_err(Error::Injected)?;
    let injected_seconds = started.elapsed();
    Ok(Collected {
        run: Run {
            video: described,
            detect: DetectSettings::from(config),
            reference: reference.detections().clone(),
            injected: injected.detections().clone(),
            targets: injector.finish(),
            provenance,
        },
        durations: Durations {
            reference: reference_seconds,
            injected: injected_seconds,
        },
    })
}

fn describe(detected: &Detected) -> Video {
    let video = detected.reading().video();
    let damage = detected.damage();
    Video {
        path: video.path().to_path_buf(),
        size: video.size(),
        rate: video.frame_rate(),
        damaged: damage.spans().to_vec(),
        reported_frames: damage.reported_frames(),
        half_window: detected.half_window(),
    }
}
