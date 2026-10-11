use std::fs;
use std::path::Path;

use serde::Serialize;
use types::span::Span;
use types::timeline::Timeline;

use crate::error::Error;
use crate::time::{hundredths, span_seconds};

#[cfg(test)]
mod tests;

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
    fs::write(path, text).map_err(|source| Error::Write {
        path: path.to_path_buf(),
        source,
    })
}
