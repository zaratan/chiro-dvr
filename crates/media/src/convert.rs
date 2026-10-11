use std::ffi::c_int;
use std::num::NonZeroU32;

use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg_next as ffmpeg;

use crate::failure::LibraryError;
use crate::frame::{self, ColorRange, YuvFrame};
use crate::{init, planes};

#[cfg(test)]
mod tests;

const RGBA_CHANNELS: usize = 4;
const LIMITED: c_int = 0;
const FULL: c_int = 1;
const NEUTRAL_BRIGHTNESS: c_int = 0;
const NEUTRAL_CONTRAST: c_int = 1 << 16;
const NEUTRAL_SATURATION: c_int = 1 << 16;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot convert a {width}×{height} image")]
    Size { width: u32, height: u32 },
    #[error("cannot set up the colour conversion")]
    Setup(#[source] LibraryError),
    #[error("this FFmpeg cannot convert with the BT.709 matrix (status {status})")]
    Matrix { status: c_int },
    #[error("a {found_width}×{found_height} frame given to a {width}×{height} conversion")]
    FrameSize {
        width: u32,
        height: u32,
        found_width: u32,
        found_height: u32,
    },
    #[error("{found} RGBA bytes given to a conversion of {expected}")]
    BufferSize { expected: usize, found: usize },
    #[error("the conversion frame cannot hold the planes")]
    FrameLayout,
    #[error("the colour conversion failed")]
    Convert(#[source] LibraryError),
    #[error("cannot shape the converted frame")]
    Frame(#[source] frame::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRange {
    Limited,
    Full,
}

/// Reads BT.709 YUV 4:2:0 in the stated range, whatever the frame's label, and writes it back in limited range.
pub struct Converter {
    to_rgba: Context,
    to_yuv: Context,
    yuv: ffmpeg::frame::Video,
    rgba: ffmpeg::frame::Video,
    width: NonZeroU32,
    height: NonZeroU32,
}

impl std::fmt::Debug for Converter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Converter")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

impl Converter {
    pub fn new(width: u32, height: u32, source: SourceRange) -> Result<Self, Error> {
        init::ffmpeg();
        let size = NonZeroU32::new(width).zip(NonZeroU32::new(height));
        let Some((nonzero_width, nonzero_height)) = size else {
            return Err(Error::Size { width, height });
        };
        let exact = Flags::ACCURATE_RND | Flags::BITEXACT;
        let context = |from, to, flags| {
            Context::get(from, width, height, to, width, height, flags | exact)
                .map_err(LibraryError::new)
                .map_err(Error::Setup)
        };
        let mut to_rgba = context(Pixel::YUV420P, Pixel::RGBA, Flags::POINT | Flags::FULL_CHR_H_INT)?;
        let mut to_yuv = context(Pixel::RGBA, Pixel::YUV420P, Flags::AREA | Flags::FULL_CHR_H_INP)?;
        let source_range = match source {
            SourceRange::Limited => LIMITED,
            SourceRange::Full => FULL,
        };
        bt709(&mut to_rgba, source_range, FULL)?;
        bt709(&mut to_yuv, FULL, LIMITED)?;
        Ok(Self {
            to_rgba,
            to_yuv,
            yuv: ffmpeg::frame::Video::new(Pixel::YUV420P, width, height),
            rgba: ffmpeg::frame::Video::new(Pixel::RGBA, width, height),
            width: nonzero_width,
            height: nonzero_height,
        })
    }

    pub fn yuv_to_rgba(&mut self, frame: &YuvFrame, rgba: &mut [u8]) -> Result<(), Error> {
        self.check_size(frame.width(), frame.height())?;
        let row = self.rgba_row();
        self.check_buffer(rgba.len())?;
        for (index, (plane, plane_row)) in frame.planes().into_iter().enumerate() {
            let stride = self.yuv.stride(index);
            let rows = plane.len() / plane_row;
            if !planes::copy_rows(plane, plane_row, self.yuv.data_mut(index), stride, plane_row, rows) {
                return Err(Error::FrameLayout);
            }
        }
        self.to_rgba
            .run(&self.yuv, &mut self.rgba)
            .map_err(LibraryError::new)
            .map_err(Error::Convert)?;
        let stride = self.rgba.stride(0);
        let rows = rgba.len() / row;
        if !planes::copy_rows(self.rgba.data(0), stride, rgba, row, row, rows) {
            return Err(Error::FrameLayout);
        }
        Ok(())
    }

    pub fn rgba_to_yuv(&mut self, rgba: &[u8], frame: &mut YuvFrame) -> Result<(), Error> {
        let row = self.rgba_row();
        self.check_buffer(rgba.len())?;
        let rows = rgba.len() / row;
        let stride = self.rgba.stride(0);
        if !planes::copy_rows(rgba, row, self.rgba.data_mut(0), stride, row, rows) {
            return Err(Error::FrameLayout);
        }
        self.to_yuv
            .run(&self.rgba, &mut self.yuv)
            .map_err(LibraryError::new)
            .map_err(Error::Convert)?;
        frame
            .reshape(self.width.get(), self.height.get(), ColorRange::Limited)
            .map_err(Error::Frame)?;
        for (index, (plane, plane_row)) in frame.planes_mut().into_iter().enumerate() {
            let stride = self.yuv.stride(index);
            let rows = plane.len() / plane_row;
            if !planes::copy_rows(self.yuv.data(index), stride, plane, plane_row, plane_row, rows) {
                return Err(Error::FrameLayout);
            }
        }
        Ok(())
    }

    fn rgba_row(&self) -> usize {
        usize::try_from(self.width.get())
            .unwrap_or(usize::MAX)
            .saturating_mul(RGBA_CHANNELS)
    }

    fn check_size(&self, found_width: u32, found_height: u32) -> Result<(), Error> {
        let (width, height) = (self.width.get(), self.height.get());
        if (found_width, found_height) == (width, height) {
            return Ok(());
        }
        Err(Error::FrameSize {
            width,
            height,
            found_width,
            found_height,
        })
    }

    fn check_buffer(&self, found: usize) -> Result<(), Error> {
        let rows = usize::try_from(self.height.get()).unwrap_or(usize::MAX);
        let expected = self.rgba_row().saturating_mul(rows);
        if found == expected {
            return Ok(());
        }
        Err(Error::BufferSize { expected, found })
    }
}

fn bt709(context: &mut Context, source_range: c_int, destination_range: c_int) -> Result<(), Error> {
    // SAFETY: SWS_CS_ITU709 is a colour space swscale knows; the call returns a pointer into its static tables.
    let coefficients = unsafe { ffmpeg::ffi::sws_getCoefficients(ffmpeg::ffi::SWS_CS_ITU709) };
    // SAFETY: the context is owned and live; nothing else borrows it while the pointer is used.
    let pointer = unsafe { context.as_mut_ptr() };
    // SAFETY: `pointer` is the live context above and `coefficients` a static four-entry table, as the call expects.
    let status = unsafe {
        ffmpeg::ffi::sws_setColorspaceDetails(
            pointer,
            coefficients,
            source_range,
            coefficients,
            destination_range,
            NEUTRAL_BRIGHTNESS,
            NEUTRAL_CONTRAST,
            NEUTRAL_SATURATION,
        )
    };
    if status < 0 {
        return Err(Error::Matrix { status });
    }
    Ok(())
}
