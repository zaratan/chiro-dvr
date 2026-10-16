use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use jiff::Zoned;
use serde::Serialize;
use tempfile::TempPath;
use types::language::Language;
use types::span::Span;
use types::timeline::Timeline;

use crate::error::Error;
use crate::names::partial_path;
use crate::time::{hundredths, span_seconds};

#[cfg(test)]
mod tests;

const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%:z";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Finish {
    Done,
    TooManyTracks,
}

#[derive(Debug, Serialize)]
pub struct Params<'a, S> {
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrackEntry {
    pub id: usize,
    pub start_s: f64,
    pub duration_s: f64,
    pub color: Option<[u8; 3]>,
    pub clip: Option<String>,
    pub zoom: Option<String>,
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
    let partial = TempPath::try_from_path(partial_path(path)).map_err(failed)?;
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
