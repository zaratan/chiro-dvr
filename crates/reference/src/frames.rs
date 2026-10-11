use std::path::Path;

use serde::{Deserialize, Serialize};
use types::detection::FrameDetections;
use types::frame::{FrameIndex, Pts};

use crate::error::Error;
use crate::lines;
use crate::measures::Measures;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameEntry {
    pub pts: Option<Pts>,
    pub key_frame: bool,
    pub decode_error: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frames {
    entries: Vec<FrameEntry>,
    detections: FrameDetections,
}

#[derive(Serialize, Deserialize)]
struct FrameLine {
    frame: usize,
    pts: Option<i64>,
    key_frame: bool,
    decode_error: bool,
    detections: Vec<Measures>,
}

impl Frames {
    pub fn new(entries: Vec<FrameEntry>, detections: FrameDetections) -> Result<Self, Error> {
        if entries.len() != detections.frame_count() {
            return Err(Error::FrameCount {
                entries: entries.len(),
                detections: detections.frame_count(),
            });
        }
        Ok(Self { entries, detections })
    }

    pub fn entries(&self) -> &[FrameEntry] {
        &self.entries
    }

    pub fn detections(&self) -> &FrameDetections {
        &self.detections
    }

    pub fn read(path: &Path) -> Result<Self, Error> {
        let lines: Vec<FrameLine> = lines::read(path)?;
        let mut entries = Vec::with_capacity(lines.len());
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
            entries.push(FrameEntry {
                pts: line.pts.map(Pts::new),
                key_frame: line.key_frame,
                decode_error: line.decode_error,
            });
            detections.push(line.detections.into_iter().map(|measures| measures.at(frame)).collect());
        }
        let detections = FrameDetections::new(detections).map_err(|source| Error::Detections {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self { entries, detections })
    }

    pub fn write(&self, path: &Path) -> Result<(), Error> {
        let lines = self
            .entries
            .iter()
            .zip(self.detections.iter())
            .map(|(entry, (frame, found))| FrameLine {
                frame: frame.get(),
                pts: entry.pts.map(Pts::get),
                key_frame: entry.key_frame,
                decode_error: entry.decode_error,
                detections: found.iter().map(Measures::of).collect(),
            });
        lines::write(path, lines)
    }

    pub fn pts(&self) -> Vec<Option<Pts>> {
        self.entries.iter().map(|entry| entry.pts).collect()
    }
}
