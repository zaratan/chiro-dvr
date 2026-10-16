use std::ffi::OsStr;
use std::fs::{self, FileType};
use std::io;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::names::{Names, Output, composed, previous_output};
use crate::table::{Headings, is_tracks_table};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub path: PathBuf,
    pub ours: bool,
}

#[derive(Debug)]
struct Found {
    path: PathBuf,
    names: Names,
    output: Output,
    kind: FileType,
}

pub fn remove_previous_outputs(
    dest: &Path,
    stem: &OsStr,
    ours: Names,
    headings: &dyn Headings,
) -> Result<Vec<Kept>, Error> {
    let (mut kept, mut removed) = (Vec::new(), Vec::new());
    for found in previous_outputs(dest, stem)? {
        if found.output == Output::TracksTable && !(found.kind.is_file() && is_tracks_table(&found.path, headings)?) {
            kept.push(Kept {
                path: found.path,
                ours: found.names == ours,
            });
        } else {
            removed.push(found);
        }
    }
    for found in &removed {
        remove(found)?;
    }
    Ok(kept)
}

fn previous_outputs(dest: &Path, stem: &OsStr) -> Result<Vec<Found>, Error> {
    let listing = |source| Error::List {
        path: dest.to_path_buf(),
        source,
    };
    let entries = match fs::read_dir(dest) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        entries => entries.map_err(listing)?,
    };
    let stem = normalized(stem);
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(listing)?;
        if let Some((names, output)) = previous_output(&normalized(&entry.file_name()), &stem) {
            let path = entry.path();
            let kind = entry.file_type().map_err(|source| Error::List {
                path: path.clone(),
                source,
            })?;
            found.push(Found {
                path,
                names,
                output,
                kind,
            });
        }
    }
    found.sort_unstable_by(|left, right| left.path.cmp(&right.path));
    Ok(found)
}

fn remove(found: &Found) -> Result<(), Error> {
    let removed = if found.kind.is_dir() {
        fs::remove_dir_all(&found.path)
    } else if is_folder_link(found.kind) {
        fs::remove_dir(&found.path)
    } else {
        fs::remove_file(&found.path)
    };
    removed.map_err(|source| Error::Remove {
        path: found.path.clone(),
        source,
    })
}

fn normalized(name: &OsStr) -> Vec<u8> {
    name.to_str()
        .map_or_else(|| name.as_encoded_bytes().to_vec(), |text| composed(text).into_bytes())
}

#[cfg(windows)]
fn is_folder_link(kind: FileType) -> bool {
    std::os::windows::fs::FileTypeExt::is_symlink_dir(&kind)
}

#[cfg(not(windows))]
const fn is_folder_link(_kind: FileType) -> bool {
    false
}
