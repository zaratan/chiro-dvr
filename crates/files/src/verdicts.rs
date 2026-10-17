use std::fs;
use std::path::Path;
use std::time::SystemTime;

use jiff::Timestamp;

use crate::names::is_hidden;

#[cfg(test)]
mod tests;

const PREFIX: &str = "verdicts";
const EXTENSION: &str = ".csv";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerdictsFile {
    pub name: String,
    pub modified: Option<String>,
    pub candidates: usize,
    pub bytes: Vec<u8>,
}

pub fn read_verdicts_file(root: &Path) -> Option<VerdictsFile> {
    let mut candidates: Vec<(SystemTime, String)> = fs::read_dir(root)
        .ok()?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter(|entry| !is_hidden(&entry.file_name()))
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let lower = name.to_lowercase();
            (lower.starts_with(PREFIX) && lower.ends_with(EXTENSION)).then_some(())?;
            let modified = entry.metadata().and_then(|metadata| metadata.modified()).ok()?;
            Some((modified, name))
        })
        .collect();
    candidates.sort();
    let count = candidates.len();
    let (modified, name) = candidates.pop()?;
    let bytes = fs::read(root.join(&name)).ok()?;
    Some(VerdictsFile {
        name,
        modified: Timestamp::try_from(modified).ok().map(|moment| moment.to_string()),
        candidates: count,
        bytes,
    })
}
