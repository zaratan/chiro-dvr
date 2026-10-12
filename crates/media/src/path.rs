use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

const PARTIAL_PREFIX: &str = ".";
const PARTIAL_SUFFIX: &str = ".partial";

pub(crate) fn for_ffmpeg(path: &Path) -> Option<&str> {
    path.to_str().filter(|text| !text.contains('\0'))
}

pub(crate) fn partial(path: &Path) -> Option<PathBuf> {
    let mut name = OsString::from(PARTIAL_PREFIX);
    name.push(path.file_name()?);
    name.push(PARTIAL_SUFFIX);
    Some(path.with_file_name(name))
}
