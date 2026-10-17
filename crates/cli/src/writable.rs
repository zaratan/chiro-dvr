use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use crate::access::Access;
use crate::jobs::Job;

#[cfg(test)]
mod tests;

const PROBE_PREFIX: &str = ".batdetect-probe-";
const CURRENT_FOLDER: &str = ".";

#[derive(Debug)]
pub struct Unwritable {
    pub folder: PathBuf,
    pub access: Access,
}

pub fn unwritable_folders(jobs: &[Job]) -> Vec<Unwritable> {
    let folders: BTreeSet<&Path> = jobs.iter().filter_map(|job| job.dest.parent()).collect();
    folders
        .into_iter()
        .filter_map(|folder| {
            probe(folder).err().map(|error| Unwritable {
                folder: folder.to_path_buf(),
                access: Access::from(error.kind()),
            })
        })
        .collect()
}

fn probe(folder: &Path) -> io::Result<()> {
    let existing = nearest_existing(folder).ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    if !existing.is_dir() {
        return Err(io::Error::from(io::ErrorKind::NotADirectory));
    }
    tempfile::Builder::new()
        .prefix(PROBE_PREFIX)
        .tempfile_in(existing)
        .and_then(tempfile::NamedTempFile::close)
}

pub(crate) fn nearest_existing(folder: &Path) -> Option<&Path> {
    folder
        .ancestors()
        .map(|ancestor| {
            if ancestor.as_os_str().is_empty() {
                Path::new(CURRENT_FOLDER)
            } else {
                ancestor
            }
        })
        .find(|ancestor| ancestor.exists())
}
