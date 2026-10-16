use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use jiff::Zoned;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Map, Value};
use types::language::Language;
use types::span::Span;
use types::timeline::Timeline;

use crate::error::Error;
use crate::names::partial_file;
use crate::time::{hundredths, span_seconds};

#[cfg(test)]
mod tests;

const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%:z";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Finish {
    Done,
    TooManyTracks,
}

#[derive(Debug, Serialize)]
pub struct Params<'a, S> {
    #[serde(serialize_with = "lossy")]
    pub video: &'a Path,
    pub fps: f64,
    #[serde(flatten)]
    pub settings: &'a S,
    pub ignored_s: Vec<[f64; 2]>,
    pub damaged_s: Vec<[f64; 2]>,
    pub damaged_frames: usize,
    pub version: &'a str,
    pub ffmpeg: &'a str,
    pub state: Finish,
    pub track_count: usize,
    pub length_s: f64,
    pub not_analysed_s: f64,
    pub elapsed_s: f64,
    pub video_bytes: u64,
    pub language: &'a str,
    pub date: String,
    pub tracks: Vec<TrackEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackEntry {
    pub id: usize,
    pub start_s: f64,
    pub duration_s: f64,
    pub color: Option<[u8; 3]>,
    pub summary: Option<String>,
    pub clip: Option<String>,
    pub zoom: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Recorded {
    pub state: Finish,
    pub version: String,
    pub video_bytes: u64,
    pub language: String,
    pub track_count: usize,
    pub length_s: f64,
    pub not_analysed_s: f64,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

fn lossy<S: Serializer>(path: &&Path, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&path.to_string_lossy())
}

pub fn read_params(path: &Path) -> Option<Recorded> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn spans_in_seconds(spans: &[Span], timeline: &Timeline) -> Vec<[f64; 2]> {
    spans
        .iter()
        .map(|&span| span_seconds(span, timeline).map(hundredths))
        .collect()
}

pub fn write_params<S: Serialize>(path: &Path, params: &Params<'_, S>) -> Result<(), Error> {
    let mut text = serde_json::to_string_pretty(params).map_err(|source| Error::Encode {
        path: path.to_path_buf(),
        source,
    })?;
    text.push('\n');
    let failed = |source| Error::Write {
        path: path.to_path_buf(),
        source,
    };
    let partial = partial_file(path).map_err(failed)?;
    let mut file = BufWriter::new(File::create(&partial).map_err(failed)?);
    file.write_all(text.as_bytes()).map_err(failed)?;
    file.flush().map_err(failed)?;
    drop(file);
    partial.persist(path).map_err(|failure| failed(failure.error))
}

pub fn language_code(language: Language) -> &'static str {
    match language {
        Language::French => "fr",
        Language::English => "en",
    }
}

pub fn local_date(time: &Zoned) -> String {
    time.strftime(DATE_FORMAT).to_string()
}
