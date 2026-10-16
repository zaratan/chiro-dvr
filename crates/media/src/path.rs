use std::ffi::{CString, OsString};
use std::fs::File;
use std::io;
use std::path::Path;

use tempfile::{Builder, NamedTempFile, TempPath};

#[cfg(test)]
mod tests;

const PARTIAL_PREFIX: &str = ".";
const PARTIAL_SUFFIX: &str = ".partial";
const UNIQUE_SEPARATOR: &str = ".";
const UNIQUE_LENGTH: usize = 6;

#[cfg(unix)]
pub(crate) fn location(path: &Path) -> Option<CString> {
    use std::os::unix::ffi::OsStrExt;
    CString::new(path.as_os_str().as_bytes()).ok()
}

#[cfg(not(unix))]
pub(crate) fn location(path: &Path) -> Option<CString> {
    path.to_str().and_then(|text| CString::new(text).ok())
}

pub(crate) fn partial(path: &Path) -> Option<io::Result<TempPath>> {
    let mut prefix = OsString::from(PARTIAL_PREFIX);
    prefix.push(path.file_name()?);
    prefix.push(UNIQUE_SEPARATOR);
    let created = Builder::new()
        .prefix(&prefix)
        .suffix(PARTIAL_SUFFIX)
        .rand_bytes(UNIQUE_LENGTH)
        .make_in(path.parent()?, |path| File::create_new(path));
    Some(created.map(NamedTempFile::into_temp_path))
}
