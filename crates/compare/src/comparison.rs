use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::Instant;

use engine::config::DetectConfig;
use engine::detection::{Detected, detect};
use engine::settings::DetectSettings;
use media::decode::Threads;
use reference::recording::Recording;
use types::detection::FrameDetections;
use types::rate::FrameRate;

use crate::agreement::Agreement;
use crate::case::Case;
use crate::error::Error;
use crate::frames::frames_of;
use crate::tracks::TrackAgreement;

#[cfg(test)]
mod tests;

pub const FRAMES_FILE: &str = "frames.jsonl";
pub const VIDEOS_AT_ONCE: usize = 6;

#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub detections: Agreement,
    pub tracks: TrackAgreement,
    pub seconds: f64,
}

pub fn compare(case: &Case, frames: Option<&Path>) -> Result<Comparison, Error> {
    let recording = Recording::read(case.recording())?;
    let video = recording.summary.video.name.clone();
    let settings: DetectSettings = recording.summary.detect_settings()?;
    let config = DetectConfig::try_from(&settings).map_err(|source| Error::Settings {
        video: video.clone(),
        source,
    })?;
    let path = case.video(&video)?;
    let start = Instant::now();
    let detected = detect(&path, Threads::Automatic, &config)?;
    let seconds = start.elapsed().as_secs_f64();
    let expected = recording.summary.background_half_window;
    if detected.half_window().get() != expected {
        return Err(Error::HalfWindow {
            video,
            expected,
            found: detected.half_window().get(),
        });
    }
    let name = case.name(&video);
    if let Some(folder) = frames {
        write_frames(&detected, &frames_path(folder, &name))?;
    }
    compared(&name, detected.detections(), &recording, seconds)
}

pub fn compare_all(cases: &[Case], at_once: usize, frames: Option<&Path>) -> Vec<Result<Comparison, Error>> {
    each_in_order(cases, at_once, |case| compare(case, frames))
}

pub fn compared(
    name: &str,
    detections: &FrameDetections,
    recording: &Recording,
    seconds: f64,
) -> Result<Comparison, Error> {
    let video = &recording.summary.video.name;
    let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).map_err(|source| Error::Rate {
        video: video.clone(),
        source,
    })?;
    Ok(Comparison {
        detections: Agreement::of(name, detections, rate, recording),
        tracks: TrackAgreement::of(name, detections, rate, recording).map_err(|source| Error::Tracking {
            video: video.clone(),
            source,
        })?,
        seconds,
    })
}

pub fn frames_path(folder: &Path, video: &str) -> PathBuf {
    let stem = Path::new(video).file_stem().unwrap_or_default();
    folder.join(stem).join(FRAMES_FILE)
}

pub(crate) fn each_in_order<T: Sync, R: Send>(items: &[T], at_once: usize, work: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = Mutex::new(items.iter().enumerate());
    let done = Mutex::new(Vec::with_capacity(items.len()));
    thread::scope(|scope| {
        for _ in 0..at_once.clamp(1, items.len().max(1)) {
            scope.spawn(|| {
                loop {
                    let taken = next.lock().unwrap_or_else(PoisonError::into_inner).next();
                    let Some((index, item)) = taken else {
                        break;
                    };
                    let result = work(item);
                    done.lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push((index, result));
                }
            });
        }
    });
    let mut done = done.into_inner().unwrap_or_else(PoisonError::into_inner);
    done.sort_by_key(|(index, _)| *index);
    done.into_iter().map(|(_, result)| result).collect()
}

fn write_frames(detected: &Detected, path: &Path) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Folder {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    Ok(frames_of(detected)?.write(path)?)
}
