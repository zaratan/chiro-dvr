use std::fmt;

use ffmpeg_next as ffmpeg;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct LibraryError(ffmpeg::Error);

impl LibraryError {
    pub(crate) fn new(error: ffmpeg::Error) -> Self {
        Self(error)
    }
}

impl fmt::Display for LibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl std::error::Error for LibraryError {}
