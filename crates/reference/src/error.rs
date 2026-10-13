use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::path::PathBuf;

use types::{rate, timeline, track};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot read {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot write {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{}:{line} does not follow the reference format", path.display())]
    Json {
        path: PathBuf,
        line: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error("cannot encode {}", path.display())]
    Encode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("{} has format version {found}, only version {supported} is read", path.display())]
    Version { path: PathBuf, found: u32, supported: u32 },
    #[error("{}:{line} holds frame {found} where frame {expected} was expected", path.display())]
    FrameOrder {
        path: PathBuf,
        line: usize,
        expected: usize,
        found: usize,
    },
    #[error("{}:{line} holds an invalid track", path.display())]
    Track {
        path: PathBuf,
        line: usize,
        #[source]
        source: track::Error,
    },
    #[error("{} does not start with the header of the tracks table", path.display())]
    CsvHeader { path: PathBuf },
    #[error("{}:{line} has {found} columns instead of {expected}", path.display())]
    CsvWidth {
        path: PathBuf,
        line: usize,
        found: usize,
        expected: usize,
    },
    #[error("{}:{line}: column {column} is not an integer", path.display())]
    CsvInteger {
        path: PathBuf,
        line: usize,
        column: &'static str,
        #[source]
        source: ParseIntError,
    },
    #[error("{}:{line}: column {column} is not a number", path.display())]
    CsvNumber {
        path: PathBuf,
        line: usize,
        column: &'static str,
        #[source]
        source: ParseFloatError,
    },
    #[error("the frame table has {entries} frames but the detections cover {detections}")]
    FrameCount { entries: usize, detections: usize },
    #[error("the recording has no valid time base")]
    Timeline(#[from] timeline::Error),
    #[error("the recording has no valid frame rate")]
    Rate(#[from] rate::Error),
    #[error("the recording has no config.DetectConfig")]
    NoDetectConfig,
    #[error("config.DetectConfig of the recording is not a detection setting")]
    DetectConfig(#[source] serde_json::Error),
    #[error("the variable {variable} does not name a folder")]
    UnsetVariable { variable: &'static str },
    #[error("the recorded video {} is not a file", path.display())]
    MissingVideo { path: PathBuf },
}
