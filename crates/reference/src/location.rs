use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use crate::error::Error;
use crate::fixture;

#[cfg(test)]
mod tests;

pub const REFERENCE_DIR_VARIABLE: &str = "BATDETECT_REFERENCE_DIR";
pub const VIDEOS_DIR_VARIABLE: &str = "BATDETECT_VIDEOS_DIR";

pub fn committed(name: &str) -> PathBuf {
    fixture::fixtures().join("reference").join(name)
}

pub fn recording_directories() -> Result<Vec<PathBuf>, Error> {
    recording_directories_under(env::var_os(REFERENCE_DIR_VARIABLE))
}

pub fn recorded_video(name: &str) -> Result<PathBuf, Error> {
    video_under(env::var_os(VIDEOS_DIR_VARIABLE), name)
}

fn folder(value: Option<OsString>, variable: &'static str) -> Result<PathBuf, Error> {
    value
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or(Error::UnsetVariable { variable })
}

fn video_under(videos: Option<OsString>, name: &str) -> Result<PathBuf, Error> {
    let path = folder(videos, VIDEOS_DIR_VARIABLE)?.join(name);
    if path.is_file() {
        Ok(path)
    } else {
        Err(Error::MissingVideo { path })
    }
}

fn recording_directories_under(root: Option<OsString>) -> Result<Vec<PathBuf>, Error> {
    let data = folder(root, REFERENCE_DIR_VARIABLE)?.join("data");
    let failed = |source| Error::Read {
        path: data.clone(),
        source,
    };
    let mut directories = Vec::new();
    for entry in fs::read_dir(&data).map_err(failed)? {
        let path = entry.map_err(failed)?.path();
        if path.is_dir() {
            directories.push(path);
        }
    }
    directories.sort();
    Ok(directories)
}
