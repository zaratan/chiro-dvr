use std::num::NonZeroU32;

use types::scale::WorkScale;
use types::size::SourceSize;

#[cfg(test)]
mod tests;

pub(crate) const MIN_WORK_SIDE: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("reducing {width}×{height} to {work_width} pixels wide is not a reduction by 3:2")]
    Reduction { width: u32, height: u32, work_width: u32 },
    #[error("a {width}×{height} work frame is narrower than {MIN_WORK_SIDE} pixels")]
    TooSmall { width: usize, height: usize },
    #[error("a {width}×{height} work frame has too many pixels to number")]
    TooLarge { width: usize, height: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Geometry {
    source_width: usize,
    width: usize,
    height: usize,
    scale: WorkScale,
}

impl Geometry {
    pub(crate) fn new(source: SourceSize, work_width: NonZeroU32) -> Result<Self, Error> {
        let (source_width, source_height) = (source.width().get(), source.height().get());
        if source_width % 3 != 0
            || source_height % 3 != 0
            || u64::from(work_width.get()) * 3 != u64::from(source_width) * 2
        {
            return Err(Error::Reduction {
                width: source_width,
                height: source_height,
                work_width: work_width.get(),
            });
        }
        let (width, height) = (work_width.get() as usize, source_height as usize / 3 * 2);
        if width < MIN_WORK_SIDE || height < MIN_WORK_SIDE {
            return Err(Error::TooSmall { width, height });
        }
        if width
            .checked_mul(height)
            .and_then(|pixels| u32::try_from(pixels).ok())
            .is_none()
        {
            return Err(Error::TooLarge { width, height });
        }
        Ok(Self {
            source_width: source_width as usize,
            width,
            height,
            scale: WorkScale::new(source, work_width),
        })
    }

    pub(crate) fn source_width(self) -> usize {
        self.source_width
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
