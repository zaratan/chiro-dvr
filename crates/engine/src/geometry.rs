use std::num::NonZeroU32;

use types::scale::WorkScale;
use types::size::SourceSize;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("a {width}×{height} work frame has too many pixels to number")]
    TooLarge { width: usize, height: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Geometry {
    source_width: usize,
    source_height: usize,
    width: usize,
    height: usize,
    scale: WorkScale,
}

impl Geometry {
    pub(crate) fn new(source: SourceSize, work_width: NonZeroU32) -> Result<Self, Error> {
        let scale = WorkScale::new(source, work_width);
        let (width, height) = (scale.work_width().get() as usize, scale.work_height().get() as usize);
        if width
            .checked_mul(height)
            .and_then(|pixels| u32::try_from(pixels).ok())
            .is_none()
        {
            return Err(Error::TooLarge { width, height });
        }
        Ok(Self {
            source_width: source.width().get() as usize,
            source_height: source.height().get() as usize,
            width,
            height,
            scale,
        })
    }

    pub(crate) fn source_width(self) -> usize {
        self.source_width
    }

    pub(crate) fn source_height(self) -> usize {
        self.source_height
    }

    pub(crate) fn width(self) -> usize {
        self.width
    }

    pub(crate) fn height(self) -> usize {
        self.height
    }

    pub(crate) fn pixels(self) -> usize {
        self.width * self.height
    }

    pub(crate) fn scale(self) -> WorkScale {
        self.scale
    }

    pub(crate) fn column(self, pixel: u32) -> usize {
        pixel as usize % self.width
    }

    pub(crate) fn row(self, pixel: u32) -> usize {
        pixel as usize / self.width
    }
}
