use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use types::detection::FrameDetections;
use types::frame::FrameIndex;

use crate::error::Error;
use crate::lines;
use crate::measures::Measures;

#[cfg(test)]
mod tests;

pub const TARGETS_FILE: &str = "bats.jsonl.gz";
pub const INJECTED_FILE: &str = "injected.jsonl.gz";
pub const REPORT_FILE: &str = "bench.json";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetLine {
    pub id: usize,
    pub motion: String,
    pub amplitude: f64,
    pub sigma: f64,
    pub start_frame: usize,
    pub positions: Vec<[f64; 2]>,
    pub amplitudes: Vec<f64>,
    pub effective: Vec<(usize, f64)>,
    pub patch: Option<Patch>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub frame: usize,
    pub base: u8,
    pub left: usize,
    pub top: usize,
    pub values: Vec<Vec<i16>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DetectionsLine {
    frame: usize,
    detections: Vec<Measures>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BenchReport {
    pub video: String,
    pub code: String,
    pub detect: Value,
    pub track: Value,
    pub stability: Value,
    #[serde(rename = "match")]
    pub matching: Value,
    pub sampling: Value,
    pub summary: Vec<ReportedClass>,
    pub not_visible: usize,
    pub false_tracks: usize,
    pub reference_tracks: usize,
    pub injected_tracks: usize,
    pub bats: Vec<ReportedTarget>,
    /// Keys the v0.1.1 does not write, kept as written.
    #[serde(flatten)]
    pub added: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportedClass {
    pub motion: String,
    pub amplitude: f64,
    pub sigma: f64,
    pub n: usize,
    pub found: usize,
    pub found_ci95: [f64; 2],
    pub median_completeness: f64,
    pub median_start_delay: Option<f64>,
    pub median_end_early: Option<f64>,
    pub mean_fragments: Option<f64>,
    pub median_effective: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportedTarget {
    pub id: usize,
    pub motion: String,
    pub amplitude: f64,
    pub sigma: f64,
    pub visible_frames: usize,
    pub found: bool,
    pub completeness: f64,
    pub start_delay: Option<usize>,
    pub end_early: Option<usize>,
    pub fragments: usize,
    pub median_effective: f64,
}

pub fn read_targets(path: &Path) -> Result<Vec<TargetLine>, Error> {
    lines::read(path)
}

pub fn write_targets(path: &Path, targets: &[TargetLine]) -> Result<(), Error> {
    lines::write(path, targets)
}

pub fn read_detections(path: &Path) -> Result<FrameDetections, Error> {
    let lines: Vec<DetectionsLine> = lines::read(path)?;
    let mut detections = Vec::with_capacity(lines.len());
    for (expected, line) in lines.into_iter().enumerate() {
        if line.frame != expected {
            return Err(Error::FrameOrder {
                path: path.to_path_buf(),
                line: expected + 1,
                expected,
                found: line.frame,
            });
        }
        let frame = FrameIndex::new(expected);
        detections.push(line.detections.into_iter().map(|measures| measures.at(frame)).collect());
    }
    FrameDetections::new(detections).map_err(|source| Error::Detections {
        path: path.to_path_buf(),
        source,
    })
}

pub fn write_detections(path: &Path, detections: &FrameDetections) -> Result<(), Error> {
    let lines = detections.iter().map(|(frame, found)| DetectionsLine {
        frame: frame.get(),
        detections: found.iter().map(Measures::of).collect(),
    });
    lines::write(path, lines)
}

pub fn read_report(path: &Path) -> Result<BenchReport, Error> {
    let text = fs::read_to_string(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| Error::Json {
        path: path.to_path_buf(),
        line: source.line(),
        source,
    })
}

pub fn write_report(path: &Path, report: &BenchReport) -> Result<(), Error> {
    let mut text = serde_json::to_string_pretty(report).map_err(|source| Error::Encode {
        path: path.to_path_buf(),
        source,
    })?;
    text.push('\n');
    fs::write(path, text).map_err(|source| Error::Write {
        path: path.to_path_buf(),
        source,
    })
}
