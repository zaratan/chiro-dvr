use std::collections::VecDeque;
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
const KEPT_PIXELS: u64 = 4_000_000;

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

struct Scratch {
    width: u32,
    height: u32,
    to_rgba: Context,
    to_yuv: Context,
    yuv: ffmpeg::frame::Video,
    rgba: ffmpeg::frame::Video,
}

pub(super) struct Regions {
    source_range: c_int,
    kept: VecDeque<Scratch>,
    kept_pixels: u64,
}

impl Regions {
    pub(super) fn new(source_range: c_int) -> Self {
        Self {
            source_range,
            kept: VecDeque::new(),
            kept_pixels: 0,
        }
    }

    fn scratch(&mut self, width: u32, height: u32) -> Result<&mut Scratch, Error> {
        let pixels = u64::from(width) * u64::from(height);
        let found = self
            .kept
            .iter()
            .position(|scratch| (scratch.width, scratch.height) == (width, height));
        let scratch = if let Some(found) = found.and_then(|index| self.kept.remove(index)) {
            self.kept_pixels -= pixels;
            found
        } else {
            let (to_rgba, to_yuv) = contexts(width, height, self.source_range)?;
            Scratch {
                width,
                height,
                to_rgba,
                to_yuv,
                yuv: ffmpeg::frame::Video::new(Pixel::YUV420P, width, height),
                rgba: ffmpeg::frame::Video::new(Pixel::RGBA, width, height),
            }
        };
        while self.kept.len() >= KEPT_SIZES || self.kept_pixels + pixels > KEPT_PIXELS {
            let Some(evicted) = self.kept.pop_front() else {
                break;
            };
            self.kept_pixels -= u64::from(evicted.width) * u64::from(evicted.height);
        }
        self.kept_pixels += pixels;
        self.kept.push_back(scratch);
        self.kept.back_mut().ok_or(Error::FrameLayout)
    }

    pub(super) fn yuv_to_rgba(
        &mut self,
        frame: &YuvFrame,
        levels: Option<&[u8; 256]>,
        rgba: &mut [u8],
        row: usize,
        region: Region,
    ) -> Result<(), Error> {
        let scratch = self.scratch(region.width, region.height)?;
        for (index, (plane, plane_row)) in frame.planes().into_iter().enumerate() {
            let InPlane { left, top, width, rows } = within(region, index);
            let stride = scratch.yuv.stride(index);
            let start = top * plane_row + left;
            let copied = plane.get(start..).is_some_and(|from| {
                planes::copy_rows(from, plane_row, scratch.yuv.data_mut(index), stride, width, rows)
            });
            if !copied {
                return Err(Error::FrameLayout);
            }
        }
        if let Some(levels) = levels {
            for level in scratch.yuv.data_mut(0) {
                *level = levels[usize::from(*level)];
            }
        }
        scratch
            .to_rgba
            .run(&scratch.yuv, &mut scratch.rgba)
            .map_err(LibraryError::new)
            .map_err(Error::Convert)?;
        let start = region.top as usize * row + region.left as usize * RGBA_CHANNELS;
        let width = region.width as usize * RGBA_CHANNELS;
        let stride = scratch.rgba.stride(0);
        let copied = rgba
            .get_mut(start..)
            .is_some_and(|to| planes::copy_rows(scratch.rgba.data(0), stride, to, row, width, region.rows()));
        if copied { Ok(()) } else { Err(Error::FrameLayout) }
    }

    pub(super) fn rgba_to_yuv(
        &mut self,
        rgba: &[u8],
        row: usize,
        frame: &mut YuvFrame,
        region: Region,
    ) -> Result<(), Error> {
        let scratch = self.scratch(region.width, region.height)?;
        let start = region.top as usize * row + region.left as usize * RGBA_CHANNELS;
        let width = region.width as usize * RGBA_CHANNELS;
        let stride = scratch.rgba.stride(0);
        let copied = rgba
            .get(start..)
            .is_some_and(|from| planes::copy_rows(from, row, scratch.rgba.data_mut(0), stride, width, region.rows()));
        if !copied {
            return Err(Error::FrameLayout);
        }
        scratch
            .to_yuv
            .run(&scratch.rgba, &mut scratch.yuv)
            .map_err(LibraryError::new)
            .map_err(Error::Convert)?;
        for (index, (plane, plane_row)) in frame.planes_mut().into_iter().enumerate() {
            let InPlane { left, top, width, rows } = within(region, index);
            let stride = scratch.yuv.stride(index);
            let start = top * plane_row + left;
            let copied = plane
                .get_mut(start..)
                .is_some_and(|to| planes::copy_rows(scratch.yuv.data(index), stride, to, plane_row, width, rows));
            if !copied {
                return Err(Error::FrameLayout);
            }
        }
        Ok(())
    }
}

struct InPlane {
    left: usize,
    top: usize,
    width: usize,
    rows: usize,
}

fn within(region: Region, plane: usize) -> InPlane {
    let shift = u32::from(plane > 0);
    InPlane {
        left: (region.left >> shift) as usize,
        top: (region.top >> shift) as usize,
        width: (region.width >> shift) as usize,
        rows: (region.height >> shift) as usize,
    }
}
