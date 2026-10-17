use std::ffi::OsStr;
use std::fs::{self, DirEntry, FileType};
use std::io;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::names::{Names, Output, is_partial_params, name_key, params_json, previous_output};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub path: PathBuf,
    pub ours: bool,
}

impl Kept {
    pub const fn blocks_our_table(&self) -> bool {
        self.ours
    }
}

#[derive(Debug)]
struct Found {
    path: PathBuf,
    names: Names,
    output: Output,
    kind: FileType,
}

pub fn remove_previous_outputs(dest: &Path, stem: &OsStr, ours: Names) -> Result<Vec<Kept>, Error> {
    remove_params(dest)?;
    let (mut kept, mut removed) = (Vec::new(), Vec::new());
    for found in previous_outputs(dest, stem)? {
        if found.output == Output::TracksTable && !found.kind.is_file() {
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

fn remove_params(dest: &Path) -> Result<(), Error> {
    remove_if_there(&params_json(dest))?;
    for entry in entries(dest)? {
        if is_partial_params(&name_key(&entry.file_name())) {
            remove_if_there(&entry.path())?;
        }
    }
    Ok(())
}

fn entries(dest: &Path) -> Result<Vec<DirEntry>, Error> {
    let listing = |source| Error::List {
        path: dest.to_path_buf(),
        source,
    };
    match fs::read_dir(dest) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        entries => entries.map_err(listing)?.map(|entry| entry.map_err(listing)).collect(),
    }
}

fn previous_outputs(dest: &Path, stem: &OsStr) -> Result<Vec<Found>, Error> {
    let stem = name_key(stem);
    let mut found = Vec::new();
    for entry in entries(dest)? {
        if let Some((names, output)) = previous_output(&name_key(&entry.file_name()), &stem) {
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

fn remove_if_there(path: &Path) -> Result<(), Error> {
    match fs::remove_file(path) {
        Err(source) if source.kind() != io::ErrorKind::NotFound => Err(Error::Remove {
            path: path.to_path_buf(),
            source,
        }),
        _ => Ok(()),
    }
}

#[cfg(windows)]
fn is_folder_link(kind: FileType) -> bool {
    std::os::windows::fs::FileTypeExt::is_symlink_dir(&kind)
}

#[cfg(not(windows))]
const fn is_folder_link(_kind: FileType) -> bool {
    false
}
