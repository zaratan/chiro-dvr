use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use jiff::Zoned;
use jiff::civil::DateTime;

use crate::error::Error;

#[cfg(test)]
mod tests;

const STAMP_FORMAT: &str = "_%Y-%m-%d_%H%M";
const MOST_IN_ONE_MINUTE: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp(DateTime);

impl Stamp {
    pub fn now() -> Self {
        Self(Zoned::now().datetime())
    }

    pub const fn at(moment: DateTime) -> Self {
        Self(moment)
    }
}

pub(crate) fn set_aside(path: &Path, stamp: Stamp) -> Result<PathBuf, Error> {
    let failed = |source| Error::SetAside {
        path: path.to_path_buf(),
        source,
    };
    let dated = reserved(path, stamp).map_err(failed)?;
    match fs::rename(path, &dated) {
        Ok(()) => Ok(dated),
        Err(source) => {
            fs::remove_file(&dated).map_err(failed)?;
            Err(failed(source))
        }
    }
}

fn reserved(path: &Path, stamp: Stamp) -> io::Result<PathBuf> {
    for rank in 1..=MOST_IN_ONE_MINUTE {
        let dated = dated_path(path, stamp, rank);
        match OpenOptions::new().write(true).create_new(true).open(&dated) {
            Ok(_) => return Ok(dated),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::from(io::ErrorKind::AlreadyExists))
}

fn dated_path(path: &Path, stamp: Stamp, rank: usize) -> PathBuf {
    let mut name = OsString::from(path.file_stem().unwrap_or_default());
    name.push(stamp.0.strftime(STAMP_FORMAT).to_string());
    if rank > 1 {
        name.push(format!("_{rank}"));
    }
    if let Some(extension) = path.extension() {
        name.push(".");
        name.push(extension);
    }
    path.with_file_name(name)
}
