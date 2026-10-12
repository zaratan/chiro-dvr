use std::ffi::{CString, OsString};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

const PARTIAL_PREFIX: &str = ".";
const PARTIAL_SUFFIX: &str = ".partial";

pub(crate) fn for_ffmpeg(path: &Path) -> Option<&str> {
    path.to_str().filter(|text| !text.contains('\0'))
}

#[cfg(unix)]
pub(crate) fn location(path: &Path) -> Option<CString> {
    use std::os::unix::ffi::OsStrExt;
    CString::new(path.as_os_str().as_bytes()).ok()
}

#[cfg(not(unix))]
pub(crate) fn location(path: &Path) -> Option<CString> {
    for_ffmpeg(path).and_then(|text| CString::new(text).ok())
}

pub(crate) fn partial(path: &Path) -> Option<PathBuf> {
    let mut name = OsString::from(PARTIAL_PREFIX);
    name.push(path.file_name()?);
    name.push(PARTIAL_SUFFIX);
    Some(path.with_file_name(name))
}
