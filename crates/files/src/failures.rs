use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::names::{hidden_dir, partial_file};

#[cfg(test)]
mod tests;

const FAILURES_FOLDER: &str = "echecs";
const EXTENSION: &str = "json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureRecord {
    pub video: String,
    pub cause: String,
    pub place: String,
    pub detail: Option<String>,
    pub date: String,
}

pub fn failures_dir(root: &Path) -> PathBuf {
    hidden_dir(root).join(FAILURES_FOLDER)
}

fn record_path(root: &Path, folder: &OsStr) -> PathBuf {
    let mut name = folder.to_os_string();
    name.push(".");
    name.push(EXTENSION);
    failures_dir(root).join(name)
}

pub fn record_failure(root: &Path, folder: &OsStr, record: &FailureRecord) -> Result<(), Error> {
    let path = record_path(root, folder);
    let failed = |source| Error::Write {
        path: path.clone(),
        source,
    };
    let dir = failures_dir(root);
    fs::create_dir_all(&dir).map_err(failed)?;
    let mut text = serde_json::to_string_pretty(record).map_err(|source| Error::Encode {
        path: path.clone(),
        source,
    })?;
    text.push('\n');
    let partial = partial_file(&path).map_err(failed)?;
    File::create(&partial)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(failed)?;
    partial.persist(&path).map_err(|failure| failed(failure.error))
}

pub fn clear_failure(root: &Path, folder: &OsStr) -> Result<(), Error> {
    let path = record_path(root, folder);
    match fs::remove_file(&path) {
        Err(source) if source.kind() != io::ErrorKind::NotFound => Err(Error::Remove { path, source }),
        _ => Ok(()),
    }
}

pub fn read_failures(root: &Path) -> Vec<(OsString, FailureRecord)> {
    let Ok(entries) = fs::read_dir(failures_dir(root)) else {
        return Vec::new();
    };
    let mut records: Vec<(OsString, FailureRecord)> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension() == Some(OsStr::new(EXTENSION))).then_some(())?;
            let record = serde_json::from_slice(&fs::read(&path).ok()?).ok()?;
            Some((path.file_stem()?.to_os_string(), record))
        })
        .collect();
    records.sort_by(|(left, _), (right, _)| left.cmp(right));
    records
}
