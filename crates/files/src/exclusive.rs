use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;

use tempfile::TempPath;

#[cfg(test)]
mod tests;

pub(crate) fn rename(partial: TempPath, path: &Path) -> io::Result<()> {
    partial.persist_noclobber(path).or_else(|failure| {
        if failure.error.kind() == io::ErrorKind::AlreadyExists {
            Err(failure.error)
        } else {
            reserved_then_renamed(failure.path, path)
        }
    })
}

pub(crate) fn reserve(path: &Path) -> io::Result<()> {
    OpenOptions::new().write(true).create_new(true).open(path).map(drop)
}

fn reserved_then_renamed(partial: TempPath, path: &Path) -> io::Result<()> {
    reserve(path)?;
    partial.persist(path).or_else(|failure| {
        fs::remove_file(path)?;
        Err(failure.error)
    })
}
