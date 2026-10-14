use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::fixture;

#[cfg(test)]
mod tests;

pub const REFERENCE_DIR_VARIABLE: &str = "BATDETECT_REFERENCE_DIR";
pub const VIDEOS_DIR_VARIABLE: &str = "BATDETECT_VIDEOS_DIR";
pub const SETTINGS_FOLDER: &str = "settings";
pub const BENCH_FOLDER: &str = "bench";
pub const OUTPUTS_FOLDER: &str = "outputs";

pub fn committed(name: &str) -> PathBuf {
    fixture::fixtures().join("reference").join(name)
}

pub fn committed_settings() -> Result<Vec<PathBuf>, Error> {
    folders_in(&committed(SETTINGS_FOLDER))
}

pub fn recording_directories() -> Result<Vec<PathBuf>, Error> {
    recording_directories_under(env::var_os(REFERENCE_DIR_VARIABLE))
}

pub fn recorded_bench(name: &str) -> Result<PathBuf, Error> {
    bench_under(env::var_os(REFERENCE_DIR_VARIABLE), name)
}

pub fn recorded_outputs(language: &str, name: &str) -> Result<PathBuf, Error> {
    outputs_under(env::var_os(REFERENCE_DIR_VARIABLE), language, name)
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

fn bench_under(root: Option<OsString>, name: &str) -> Result<PathBuf, Error> {
    Ok(folder(root, REFERENCE_DIR_VARIABLE)?.join(BENCH_FOLDER).join(name))
}

fn outputs_under(root: Option<OsString>, language: &str, name: &str) -> Result<PathBuf, Error> {
    Ok(folder(root, REFERENCE_DIR_VARIABLE)?
        .join(OUTPUTS_FOLDER)
        .join(language)
        .join(name))
}

fn recording_directories_under(root: Option<OsString>) -> Result<Vec<PathBuf>, Error> {
    folders_in(&folder(root, REFERENCE_DIR_VARIABLE)?.join("data"))
}

fn folders_in(parent: &Path) -> Result<Vec<PathBuf>, Error> {
    let failed = |source| Error::Read {
        path: parent.to_path_buf(),
        source,
    };
    let mut directories = Vec::new();
    for entry in fs::read_dir(parent).map_err(failed)? {
        let path = entry.map_err(failed)?.path();
        if path.is_dir() {
            directories.push(path);
        }
    }
    directories.sort();
    Ok(directories)
}
