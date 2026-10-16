use std::ffi::OsStr;
use std::fs::{self, FileType};
use std::io;
use std::path::{Path, PathBuf};

use crate::annotation::carries_verdicts;
use crate::aside::{Stamp, set_aside};
use crate::error::Error;
use crate::names::{Names, Output, composed, previous_output};
use crate::table::{Headings, is_tracks_table};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Verdicts,
    UnknownForm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kept {
    Left {
        path: PathBuf,
        ours: bool,
    },
    SetAside {
        path: PathBuf,
        dated: PathBuf,
        reason: Reason,
    },
}

impl Kept {
    pub const fn blocks_our_table(&self) -> bool {
        matches!(self, Self::Left { ours: true, .. })
    }
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
    stamp: Stamp,
) -> Result<Vec<Kept>, Error> {
    let (mut kept, mut aside, mut removed) = (Vec::new(), Vec::new(), Vec::new());
    for found in previous_outputs(dest, stem)? {
        if found.output != Output::TracksTable {
            removed.push(found);
        } else if !found.kind.is_file() {
            kept.push(Kept::Left {
                path: found.path,
                ours: found.names == ours,
            });
        } else {
            match foreign(&found.path, headings)? {
                Some(reason) => aside.push((found.path, reason)),
                None => removed.push(found),
            }
        }
    }
    for (path, reason) in aside {
        let dated = set_aside(&path, stamp)?;
        kept.push(Kept::SetAside { path, dated, reason });
    }
    for found in &removed {
        remove(found)?;
    }
    Ok(kept)
}

fn foreign(path: &Path, headings: &dyn Headings) -> Result<Option<Reason>, Error> {
    let bytes = fs::read(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(if is_tracks_table(&bytes, headings) {
        None
    } else if carries_verdicts(&bytes, headings) {
        Some(Reason::Verdicts)
    } else {
        Some(Reason::UnknownForm)
    })
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
