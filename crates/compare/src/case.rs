use std::path::{Path, PathBuf};

use reference::fixture::fixture;
use reference::location::{committed, committed_settings, recorded_video, recording_directories};

use crate::error::Error;

#[cfg(test)]
mod tests;

pub const EXTRACT: &str = "video_092_original_3m24-4m05";
pub const WHOLE: &str = "video_092_original";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Extract,
    Whole,
    Settings,
    Recorded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    recording: PathBuf,
    place: Place,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Place {
    Fixtures,
    Settings,
    Recordings,
}

impl Target {
    pub fn tolerates_refusals(self) -> bool {
        self == Self::Settings
    }

    pub fn cases(self) -> Result<Vec<Case>, Error> {
        Ok(match self {
            Self::Extract => vec![Case::committed(EXTRACT)],
            Self::Whole => vec![Case::committed(WHOLE)],
            Self::Settings => committed_settings()?.into_iter().map(Case::setting).collect(),
            Self::Recorded => recording_directories()?.into_iter().map(Case::recorded).collect(),
        })
    }
}

impl Case {
    pub fn committed(name: &str) -> Self {
        Self {
            recording: committed(name),
            place: Place::Fixtures,
        }
    }

    pub fn setting(recording: PathBuf) -> Self {
        Self {
            recording,
            place: Place::Settings,
        }
    }

    pub fn recorded(recording: PathBuf) -> Self {
        Self {
            recording,
            place: Place::Recordings,
        }
    }

    pub fn recording(&self) -> &Path {
        &self.recording
    }

    pub fn name(&self, video: &str) -> String {
        match (self.place, self.recording.file_name()) {
            (Place::Settings, Some(folder)) => folder.to_string_lossy().into_owned(),
            _ => video.to_owned(),
        }
    }

    pub fn video(&self, name: &str) -> Result<PathBuf, Error> {
        Ok(match self.place {
            Place::Fixtures | Place::Settings => fixture(name)?,
            Place::Recordings => recorded_video(name)?,
        })
    }
}
