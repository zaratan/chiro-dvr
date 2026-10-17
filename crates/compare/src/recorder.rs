use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{self, Component, Path, PathBuf};

use engine::config::DetectConfig;
use engine::detection::detect_input;
use engine::input::Input;
use engine::settings::DetectSettings;
use media::decode::Threads;
use reference::fixture::fixtures;
use reference::frames::Frames;
use reference::location::{REFERENCE_DIR_VARIABLE, REFERENCE_V020_DIR_VARIABLE};
use reference::recording::{Recording, SUMMARY_FILE};
use reference::video::VideoSummary;

use crate::case::Case;
use crate::comparison::{each_in_order, frames_path};
use crate::decoded::{followed, same_stream, summary};
use crate::error::Error;
use crate::frames::frames_of;
use crate::provenance::Build;
use crate::sameness::Sameness;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Recorded {
    pub name: String,
    pub directory: PathBuf,
    pub summary: VideoSummary,
    pub same_as: Option<Sameness>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Places<'a> {
    pub out: &'a Path,
    pub same_as: Option<&'a Path>,
}

pub fn record(case: &Case, build: &Build, places: &Places<'_>) -> Result<Recorded, Error> {
    let probed = VideoSummary::read(&case.recording().join(SUMMARY_FILE))?;
    let video = probed.video.name.clone();
    let settings: DetectSettings = probed.detect_settings()?;
    let config = DetectConfig::try_from(&settings).map_err(|source| Error::Settings {
        video: video.clone(),
        source,
    })?;
    let name = case.name(&video);
    let previous = places
        .same_as
        .map(|folder| Frames::read(&frames_path(folder, &name)))
        .transpose()?;
    let input = Input::open(&case.video(&video)?, Threads::Automatic).map_err(engine::detection::Error::from)?;
    same_stream(&probed, input.video())?;
    let detected = detect_input(input, &config)?;
    if detected.half_window().get() != probed.background_half_window {
        return Err(Error::HalfWindow {
            video,
            expected: probed.background_half_window,
            found: detected.half_window().get(),
        });
    }
    let followed = followed(&detected).map_err(|source| Error::Tracking {
        video: video.clone(),
        source,
    })?;
    let summary = summary(probed, &detected, &followed, &config, build)?;
    let recording = Recording {
        summary,
        frames: frames_of(&detected)?,
        tracks: followed.tracks,
        rows: None,
    };
    let directory = places.out.join(case.folder());
    fs::create_dir_all(&directory).map_err(|source| Error::Folder {
        path: directory.clone(),
        source,
    })?;
    recording.write(&directory)?;
    let same_as =
        previous.map(|previous| Sameness::of(&name, &previous, &recording.frames, &recording.summary.damaged_spans));
    Ok(Recorded {
        name,
        directory,
        summary: recording.summary,
        same_as,
    })
}

/// The folders `out` must not reach into: the committed references, those named by their variables, and `same_as`.
pub fn protected_folders(same_as: Option<&Path>) -> Vec<PathBuf> {
    let variables = [REFERENCE_DIR_VARIABLE, REFERENCE_V020_DIR_VARIABLE]
        .into_iter()
        .filter_map(env::var_os)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    [fixtures()]
        .into_iter()
        .chain(variables)
        .chain(same_as.map(Path::to_path_buf))
        .collect()
}

pub fn refuse_overlap(out: &Path, protected: &[PathBuf]) -> Result<(), Error> {
    let out = resolved(out).map_err(|source| Error::Folder {
        path: out.to_path_buf(),
        source,
    })?;
    match protected
        .iter()
        .filter_map(|folder| fs::canonicalize(folder).ok())
        .find(|folder| out.starts_with(folder) || folder.starts_with(&out))
    {
        Some(protected) => Err(Error::Overlap { out, protected }),
        None => Ok(()),
    }
}

fn resolved(path: &Path) -> io::Result<PathBuf> {
    let mut existing = path::absolute(path)?;
    let mut missing = Vec::new();
    let found = loop {
        match fs::canonicalize(&existing) {
            Ok(found) => break found,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let Some(last) = existing.components().next_back() else {
                    return Err(error);
                };
                missing.push(last.as_os_str().to_owned());
                if !existing.pop() {
                    return Err(error);
                }
            }
            Err(error) => return Err(error),
        }
    };
    Ok(missing.iter().rev().fold(found, |mut resolved, part| {
        match Path::new(part).components().next() {
            Some(Component::ParentDir) => {
                resolved.pop();
            }
            Some(Component::CurDir) | None => {}
            Some(_) => resolved.push(part),
        }
        resolved
    }))
}

pub fn holds(answers: &[Result<Recorded, Error>]) -> bool {
    !answers.is_empty() && answers.iter().all(|answer| answer.as_ref().is_ok_and(Recorded::holds))
}

pub fn record_all(cases: &[Case], at_once: usize, build: &Build, places: &Places<'_>) -> Vec<Result<Recorded, Error>> {
    each_in_order(cases, at_once, |case| record(case, build, places))
}

impl Recorded {
    pub fn holds(&self) -> bool {
        self.same_as.as_ref().is_none_or(Sameness::holds)
    }

    pub fn write_line(&self, out: &mut impl Write) -> io::Result<()> {
        let counts = &self.summary.counts;
        let spans = &self.summary.damaged_spans;
        writeln!(
            out,
            "{}\tframes {}\tdetections {}\ttracks {}\tdamaged spans {} ({} frames)\t{}",
            self.name,
            counts.frames,
            counts.detections,
            counts.tracks,
            spans.len(),
            spans.iter().map(|span| span.frame_count()).sum::<usize>(),
            self.directory.display()
        )?;
        match &self.same_as {
            Some(sameness) => sameness.write_line(out),
            None => Ok(()),
        }
    }
}
