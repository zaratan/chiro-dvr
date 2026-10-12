use ffmpeg_next as ffmpeg;
use types::frame::Pts;
use types::rational::Rational;

use super::{Decoder, Error, StreamInfo};
use crate::failure::LibraryError;
use crate::frame::YuvFrame;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
#[error("cannot seek to the key frame before pts {pts}")]
pub struct SeekError {
    pts: i64,
    #[source]
    source: LibraryError,
}

#[derive(Debug)]
pub struct Placed {
    decoder: Decoder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacedFrame {
    pts: Option<Pts>,
}

impl PlacedFrame {
    pub fn pts(self) -> Option<Pts> {
        self.pts
    }
}

impl Decoder {
    /// Lands on the last key frame at or before `pts` that the container index knows.
    pub fn seek_before(mut self, pts: Pts) -> Result<Placed, SeekError> {
        let target = microseconds(pts, self.info.time_base);
        self.input.seek(target, ..target).map_err(|source| SeekError {
            pts: pts.get(),
            source: LibraryError::new(source),
        })?;
        self.codec.flush();
        Ok(Placed { decoder: self })
    }

    pub fn at_start(self) -> Placed {
        Placed { decoder: self }
    }
}

impl Placed {
    pub fn stream(&self) -> &StreamInfo {
        self.decoder.stream()
    }

    pub fn next(&mut self, into: &mut YuvFrame) -> Result<Option<PlacedFrame>, Error> {
        let decoded = self.decoder.next(into)?;
        Ok(decoded.map(|info| PlacedFrame { pts: info.pts() }))
    }
}

fn microseconds(pts: Pts, time_base: Rational) -> i64 {
    let scaled = i128::from(pts.get()) * i128::from(time_base.num()) * i128::from(ffmpeg::ffi::AV_TIME_BASE);
    let floored = scaled.div_euclid(i128::from(time_base.den()));
    i64::try_from(floored).unwrap_or(if floored < 0 { i64::MIN } else { i64::MAX })
}
