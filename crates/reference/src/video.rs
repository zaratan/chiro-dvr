use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use types::frame::FrameIndex;
use types::rational::Rational;
use types::span::Span;

use crate::error::Error;
use crate::fields;

#[cfg(test)]
mod tests;

pub const PROBED_FORMAT: u32 = 1;
pub const DECODED_FORMAT: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoSummary {
    pub format_version: u32,
    pub reference: String,
    pub video: VideoFile,
    pub stream: Stream,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ffprobe: Option<Probe>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decoder: Option<Decoder>,
    pub background_half_window: usize,
    #[serde(with = "fields::spans")]
    pub damaged_spans: Vec<Span>,
    #[serde(with = "fields::spans")]
    pub unstable_spans: Vec<Span>,
    pub counts: Counts,
    pub files: Files,
    /// The settings, versions and other untyped sections of the recording, kept as written.
    #[serde(flatten)]
    pub recorded: Map<String, Value>,
}

#[derive(Deserialize)]
struct Version {
    format_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoFile {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stream {
    pub width: u32,
    pub height: u32,
    pub r_frame_rate: Rational,
    pub avg_frame_rate: Rational,
    pub time_base: Rational,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_frames: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Probe {
    pub frame_count: usize,
    pub error_frames: Vec<FrameIndex>,
    pub gap_frames: Vec<FrameIndex>,
    pub decode_messages: Vec<DecodeMessages>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decoder {
    pub error_frames: Vec<FrameIndex>,
    pub gap_frames: Vec<FrameIndex>,
    pub losses: Losses,
    pub threads: DecoderThreads,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "count", rename_all = "snake_case")]
pub enum DecoderThreads {
    Single,
    Frame(NonZeroUsize),
    Slice(NonZeroUsize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Losses {
    pub before_first: usize,
    pub after_last: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeMessages {
    pub frame: FrameIndex,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub frames: usize,
    pub detections: usize,
    pub detections_kept: usize,
    pub damaged_frames_reported: usize,
    pub tracks: usize,
    pub track_points: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Files {
    pub frames: String,
    pub tracks: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub csv: Option<String>,
}

impl VideoSummary {
    pub fn read(path: &Path) -> Result<Self, Error> {
        let text = fs::read_to_string(path).map_err(|source| Error::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let malformed = |source: serde_json::Error| Error::Json {
            path: path.to_path_buf(),
            line: source.line(),
            source,
        };
        let version: Version = serde_json::from_str(&text).map_err(malformed)?;
        if ![PROBED_FORMAT, DECODED_FORMAT].contains(&version.format_version) {
            return Err(Error::Version {
                path: path.to_path_buf(),
                found: version.format_version,
            });
        }
        let summary: Self = serde_json::from_str(&text).map_err(malformed)?;
        summary.sections_of_its_version(path)?;
        Ok(summary)
    }

    fn sections_of_its_version(&self, path: &Path) -> Result<(), Error> {
        let probed = self.format_version == PROBED_FORMAT;
        let sections = [
            ("ffprobe", self.ffprobe.is_some(), probed),
            ("decoder", self.decoder.is_some(), !probed),
            ("files.csv", self.files.csv.is_some(), probed),
        ];
        match sections.into_iter().find(|(_, present, wanted)| present != wanted) {
            Some((section, _, true)) => Err(Error::MissingSection {
                path: path.to_path_buf(),
                version: self.format_version,
                section,
            }),
            Some((section, _, false)) => Err(Error::ForeignSection {
                path: path.to_path_buf(),
                version: self.format_version,
                section,
            }),
            None => Ok(()),
        }
    }

    pub fn detect_settings<T: DeserializeOwned>(&self) -> Result<T, Error> {
        let recorded = self
            .recorded
            .get("config")
            .and_then(|config| config.get("DetectConfig"))
            .ok_or(Error::NoDetectConfig)?;
        T::deserialize(recorded).map_err(Error::DetectConfig)
    }

    pub fn write(&self, path: &Path) -> Result<(), Error> {
        self.sections_of_its_version(path)?;
        let mut text = serde_json::to_string_pretty(self).map_err(|source| Error::Encode {
            path: path.to_path_buf(),
            source,
        })?;
        text.push('\n');
        fs::write(path, text).map_err(|source| Error::Write {
            path: path.to_path_buf(),
            source,
        })
    }
}
