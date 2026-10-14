use std::fmt;
use std::io;

use ffmpeg_next as ffmpeg;
use ffmpeg_next::util::error::{EACCES, ENOENT, ENOSPC, EPERM, EROFS};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct LibraryError(ffmpeg::Error);

impl LibraryError {
    pub(crate) fn new(error: ffmpeg::Error) -> Self {
        Self(error)
    }

    /// The kind of the system error behind it, read from the errno that FFmpeg carries (CRT errno on Windows).
    pub fn io_kind(&self) -> Option<io::ErrorKind> {
        let ffmpeg::Error::Other { errno } = self.0 else {
            return None;
        };
        match errno {
            ENOENT => Some(io::ErrorKind::NotFound),
            EACCES | EPERM => Some(io::ErrorKind::PermissionDenied),
            ENOSPC => Some(io::ErrorKind::StorageFull),
            EROFS => Some(io::ErrorKind::ReadOnlyFilesystem),
            _ => None,
        }
    }
}

impl fmt::Display for LibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl std::error::Error for LibraryError {}
