use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tempfile::TempPath;

use crate::error::Error;
use crate::names::{hidden_dir, partial_path};

#[cfg(test)]
mod tests;

const REFUSALS_FILE: &str = "refus.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefusalRecord {
    pub date: String,
    pub inputs: Vec<RefusedInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefusedInput {
    pub path: String,
    pub reason: String,
    pub cause: Option<String>,
    pub todo: String,
    pub videos: Vec<String>,
    pub detail: Option<String>,
}

pub fn refusals_path(root: &Path) -> PathBuf {
    hidden_dir(root).join(REFUSALS_FILE)
}

pub fn record_refusals(root: &Path, record: &RefusalRecord) -> Result<(), Error> {
    let path = refusals_path(root);
    let failed = |source| Error::Write {
        path: path.clone(),
        source,
    };
    fs::create_dir_all(hidden_dir(root)).map_err(failed)?;
    let mut text = serde_json::to_string_pretty(record).map_err(|source| Error::Encode {
        path: path.clone(),
        source,
    })?;
    text.push('\n');
    let partial = TempPath::try_from_path(partial_path(&path)).map_err(failed)?;
    File::create(&partial)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(failed)?;
    partial.persist(&path).map_err(|failure| failed(failure.error))
}

pub fn clear_refusals(root: &Path) -> Result<(), Error> {
    let path = refusals_path(root);
    match fs::remove_file(&path) {
        Err(source) if source.kind() != io::ErrorKind::NotFound => Err(Error::Remove { path, source }),
        _ => Ok(()),
    }
}

pub fn read_refusals(root: &Path) -> Option<RefusalRecord> {
    let record: RefusalRecord = serde_json::from_slice(&fs::read(refusals_path(root)).ok()?).ok()?;
    (!record.inputs.is_empty()).then_some(record)
}
