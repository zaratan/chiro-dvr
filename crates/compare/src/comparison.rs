use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::Instant;

use engine::config::DetectConfig;
use engine::detection::detect;
use engine::settings::DetectSettings;
use media::decode::Threads;
use reference::frames::Frames;
use reference::recording::Recording;
use types::rate::FrameRate;
use types::span::Span;

use crate::agreement::Agreement;
use crate::case::Case;
use crate::damaged::{joined, merged, reported_by};
use crate::error::Error;
use crate::frames::{Pass, pass_of};
use crate::inputs::{DecodeErrors, InputGap, REFERENCE_FIELDS, decode_errors, entry_gap, losses_gap};
use crate::sameness::Sameness;
use crate::tracks::TrackAgreement;

#[cfg(test)]
mod tests;

pub const FRAMES_FILE: &str = "frames.jsonl";
pub const VIDEOS_AT_ONCE: usize = 6;

#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub decode_errors: DecodeErrors,
    pub ignored: Vec<Span>,
    pub detections: Agreement,
    pub tracks: TrackAgreement,
    pub seconds: f64,
    pub same_as: Option<Sameness>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Folders {
    pub frames: Option<PathBuf>,
    pub same_as: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gaps {
    pub differing_frames: usize,
    pub lost_detections: usize,
    pub added_detections: usize,
    pub paired_tracks: usize,
    pub agreeing_tracks: usize,
    pub lost_tracks: usize,
    pub added_tracks: usize,
}

impl Gaps {
    pub fn agreeing(tracks: usize) -> Self {
        Self {
            differing_frames: 0,
            lost_detections: 0,
            added_detections: 0,
            paired_tracks: tracks,
            agreeing_tracks: tracks,
            lost_tracks: 0,
            added_tracks: 0,
        }
    }
}

impl Comparison {
    pub fn differs(&self) -> bool {
        self.detections.differs() || self.tracks.differs()
    }

    pub fn gaps(&self) -> Gaps {
        let detections = self.detections.counts();
        let tracks = self.tracks.counts();
        Gaps {
            differing_frames: detections.differing_frames,
            lost_detections: detections.lost,
            added_detections: detections.added,
            paired_tracks: tracks.paired,
            agreeing_tracks: tracks.agreeing,
            lost_tracks: tracks.lost,
            added_tracks: tracks.added,
        }
    }
}

pub fn compare(case: &Case, folders: &Folders) -> Result<Comparison, Error> {
    let recording = Recording::read(case.recording())?;
    let video = recording.summary.video.name.clone();
    let settings: DetectSettings = recording.summary.detect_settings()?;
    let config = DetectConfig::try_from(&settings).map_err(|source| Error::Settings {
        video: video.clone(),
        source,
    })?;
    let name = case.name(&video);
    let previous = folders
        .same_as
        .as_deref()
        .map(|folder| Frames::read(&frames_path(folder, &name)))
        .transpose()?;
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
    let current = pass_of(&detected)?;
    if let Some(folder) = &folders.frames {
        write_frames(&current.frames, &frames_path(folder, &name))?;
    }
    let mut comparison = compared(&name, &current, &recording, seconds)?;
    comparison.same_as =
        previous.map(|previous| Sameness::of(&name, &previous, &current.frames, detected.damage().spans()));
    Ok(comparison)
}

pub fn compare_all(cases: &[Case], at_once: usize, folders: &Folders) -> Vec<Result<Comparison, Error>> {
    each_in_order(cases, at_once, |case| compare(case, folders))
}

pub fn compared(name: &str, current: &Pass, recording: &Recording, seconds: f64) -> Result<Comparison, Error> {
    let summary = &recording.summary;
    let video = &summary.video.name;
    let rate = FrameRate::new(summary.stream.avg_frame_rate).map_err(|source| Error::Rate {
        video: video.clone(),
        source,
    })?;
    let entries = recording.frames.entries();
    if let Some(gap) = input_gap(recording, current) {
        return Err(Error::Inputs {
            video: video.clone(),
            gap,
        });
    }
    let damage = joined(
        &reported_by(summary),
        &current.reports,
        current.frames.entries(),
        summary.background_half_window,
    );
    let ignored = merged(summary.damaged_spans.iter().chain(damage.spans()).copied());
    let detections = current.frames.detections();
    Ok(Comparison {
        decode_errors: decode_errors(entries, current.frames.entries()),
        detections: Agreement::of(name, detections, rate, recording, &ignored),
        tracks: TrackAgreement::of(name, detections, rate, recording, &damage).map_err(|source| Error::Tracking {
            video: video.clone(),
            source,
        })?,
        ignored,
        seconds,
        same_as: None,
    })
}

fn input_gap(recording: &Recording, current: &Pass) -> Option<InputGap> {
    entry_gap(recording.frames.entries(), current.frames.entries(), &REFERENCE_FIELDS).or_else(|| {
        recording
            .summary
            .decoder
            .as_ref()
            .and_then(|decoder| losses_gap(decoder.losses, current.reports.edges))
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

fn write_frames(frames: &Frames, path: &Path) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Folder {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    Ok(frames.write(path)?)
}
