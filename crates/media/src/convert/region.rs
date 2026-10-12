use std::ffi::c_int;

use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::Context;
use ffmpeg_next as ffmpeg;

use super::{Error, RGBA_CHANNELS, contexts};
use crate::failure::LibraryError;
use crate::frame::YuvFrame;
use crate::planes;

#[cfg(test)]
mod tests;

const KEPT_SIZES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub(super) fn fits(self, width: u32, height: u32) -> bool {
        let even = [self.left, self.top, self.width, self.height]
            .iter()
            .all(|value| value % 2 == 0);
        let inside = self.left.checked_add(self.width).is_some_and(|right| right <= width)
            && self.top.checked_add(self.height).is_some_and(|bottom| bottom <= height);
        even && self.width > 0 && self.height > 0 && inside
    }

    fn rows(self) -> usize {
        self.height as usize
    }
}

struct Sized {
    width: u32,
    height: u32,
    to_rgba: Context,
    to_yuv: Context,
    yuv: ffmpeg::frame::Video,
    rgba: ffmpeg::frame::Video,
}

pub(super) struct Regions {
    source_range: c_int,
    sized: Vec<Sized>,
}

impl Regions {
    pub(super) fn new(source_range: c_int) -> Self {
        Self {
            source_range,
            sized: Vec::new(),
        }
    }

    fn sized(&mut self, width: u32, height: u32) -> Result<&mut Sized, Error> {
        let found = self
            .sized
            .iter()
            .position(|sized| (sized.width, sized.height) == (width, height));
        let index = if let Some(index) = found {
            index
        } else {
            if self.sized.len() == KEPT_SIZES {
                self.sized.remove(0);
            }
            let (to_rgba, to_yuv) = contexts(width, height, self.source_range)?;
            self.sized.push(Sized {
                width,
                height,
                to_rgba,
                to_yuv,
                yuv: ffmpeg::frame::Video::new(Pixel::YUV420P, width, height),
                rgba: ffmpeg::frame::Video::new(Pixel::RGBA, width, height),
            });
            self.sized.len() - 1
        };
        self.sized.get_mut(index).ok_or(Error::FrameLayout)
    }

    pub(super) fn yuv_to_rgba(
        &mut self,
        frame: &YuvFrame,
        levels: Option<&[u8; 256]>,
        rgba: &mut [u8],
        row: usize,
        region: Region,
    ) -> Result<(), Error> {
        let sized = self.sized(region.width, region.height)?;
        for (index, (plane, plane_row)) in frame.planes().into_iter().enumerate() {
            let (left, top, width, rows) = within(region, index);
            let stride = sized.yuv.stride(index);
            let start = top * plane_row + left;
            let copied = plane
                .get(start..)
                .is_some_and(|from| planes::copy_rows(from, plane_row, sized.yuv.data_mut(index), stride, width, rows));
            if !copied {
                return Err(Error::FrameLayout);
            }
        }
        if let Some(levels) = levels {
            for level in sized.yuv.data_mut(0) {
                *level = levels[usize::from(*level)];
            }
        }
        sized
            .to_rgba
            .run(&sized.yuv, &mut sized.rgba)
            .map_err(LibraryError::new)
            .map_err(Error::Convert)?;
        let start = region.top as usize * row + region.left as usize * RGBA_CHANNELS;
        let width = region.width as usize * RGBA_CHANNELS;
        let stride = sized.rgba.stride(0);
        let copied = rgba
            .get_mut(start..)
            .is_some_and(|to| planes::copy_rows(sized.rgba.data(0), stride, to, row, width, region.rows()));
        if copied { Ok(()) } else { Err(Error::FrameLayout) }
    }

    pub(super) fn rgba_to_yuv(
        &mut self,
        rgba: &[u8],
        row: usize,
        frame: &mut YuvFrame,
        region: Region,
    ) -> Result<(), Error> {
        let sized = self.sized(region.width, region.height)?;
        let start = region.top as usize * row + region.left as usize * RGBA_CHANNELS;
        let width = region.width as usize * RGBA_CHANNELS;
        let stride = sized.rgba.stride(0);
        let copied = rgba
            .get(start..)
            .is_some_and(|from| planes::copy_rows(from, row, sized.rgba.data_mut(0), stride, width, region.rows()));
        if !copied {
            return Err(Error::FrameLayout);
        }
        sized
            .to_yuv
            .run(&sized.rgba, &mut sized.yuv)
            .map_err(LibraryError::new)
            .map_err(Error::Convert)?;
        for (index, (plane, plane_row)) in frame.planes_mut().into_iter().enumerate() {
            let (left, top, width, rows) = within(region, index);
            let stride = sized.yuv.stride(index);
            let start = top * plane_row + left;
            let copied = plane
                .get_mut(start..)
                .is_some_and(|to| planes::copy_rows(sized.yuv.data(index), stride, to, plane_row, width, rows));
            if !copied {
                return Err(Error::FrameLayout);
            }
        }
        Ok(())
    }
}

fn within(region: Region, plane: usize) -> (usize, usize, usize, usize) {
    let shift = u32::from(plane > 0);
    (
        (region.left >> shift) as usize,
        (region.top >> shift) as usize,
        (region.width >> shift) as usize,
        (region.height >> shift) as usize,
    )
}
