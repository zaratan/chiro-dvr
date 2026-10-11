use std::fmt;
use std::mem;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use ffmpeg::codec::threading;
use ffmpeg::format::Pixel;
use ffmpeg::util::color::Range;
use ffmpeg::util::error::EAGAIN;
use ffmpeg_next as ffmpeg;
use types::rational::Rational;

use crate::failure::LibraryError;
use crate::frame::{self, ColorRange, YuvFrame};
use crate::{init, path, planes};

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("FFmpeg cannot open {}: the path is not UTF-8 or holds a NUL byte", path.display())]
    UnusablePath { path: PathBuf },
    #[error("cannot open {}", path.display())]
    Open {
        path: PathBuf,
        #[source]
        source: LibraryError,
    },
    #[error("{} has no video stream", .0.display())]
    NoVideoStream(PathBuf),
    #[error("the video stream has no usable frame rate")]
    UnknownFrameRate,
    #[error("the video stream has no usable time base")]
    UnknownTimeBase,
    #[error("frames in {format} are not handled, only 8-bit 4:2:0")]
    UnsupportedPixelFormat { format: &'static str },
    #[error("a {found_width}×{found_height} frame in a {width}×{height} stream")]
    SizeChanged {
        width: u32,
        height: u32,
        found_width: u32,
        found_height: u32,
    },
    #[error("the decoder handed back a frame shorter than its size")]
    TruncatedFrame,
    #[error("the video stream has an unusable frame size")]
    Size(#[source] frame::Error),
    #[error("cannot read the next packet")]
    Read(#[source] LibraryError),
    #[error("the decoder failed")]
    Decode(#[source] LibraryError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamInfo {
    width: NonZeroU32,
    height: NonZeroU32,
    frame_rate: Rational,
    time_base: Rational,
    range: ColorRange,
}

impl StreamInfo {
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    pub fn height(&self) -> u32 {
        self.height.get()
    }

    pub fn frame_rate(&self) -> Rational {
        self.frame_rate
    }

    pub fn time_base(&self) -> Rational {
        self.time_base
    }

    pub fn range(&self) -> ColorRange {
        self.range
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct FrameInfo {
    key: bool,
    unreadable_packets: u32,
}

impl FrameInfo {
    pub fn key(self) -> bool {
        self.key
    }

    pub fn unreadable_packets(self) -> u32 {
        self.unreadable_packets
    }
}

/// Unreadable packets are counted, not fatal; after an `Err` the decoder is unusable.
pub struct Decoder {
    input: ffmpeg::format::context::Input,
    stream: usize,
    codec: ffmpeg::decoder::Video,
    decoded: ffmpeg::frame::Video,
    info: StreamInfo,
    unreadable: u32,
}

impl fmt::Debug for Decoder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Decoder")
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

impl Decoder {
    pub fn open(path: &Path) -> Result<Self, Error> {
        init::ffmpeg();
        let unusable = || Error::UnusablePath {
            path: path.to_path_buf(),
        };
        let location = path::for_ffmpeg(path).ok_or_else(unusable)?;
        let failed = |source| Error::Open {
            path: path.to_path_buf(),
            source: LibraryError::new(source),
        };
        let input = ffmpeg::format::input(location).map_err(failed)?;
        let best = input.streams().best(ffmpeg::media::Type::Video);
        let stream = best.ok_or_else(|| Error::NoVideoStream(path.to_path_buf()))?;
        let index = stream.index();
        let frame_rate = frame_rate(stream.avg_frame_rate(), stream.rate())?;
        let time_base = positive(stream.time_base()).ok_or(Error::UnknownTimeBase)?;
        let mut context = ffmpeg::codec::context::Context::from_parameters(stream.parameters()).map_err(failed)?;
        context.set_threading(threading::Config::kind(threading::Type::Frame));
        let codec = context.decoder().video().map_err(failed)?;
        let (width, height) = (codec.width(), codec.height());
        let (Some(nonzero_width), Some(nonzero_height)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
            return Err(Error::Size(frame::Error::EmptySize { width, height }));
        };
        let range = color_range(codec.format(), codec.color_range())?;
        let info = StreamInfo {
            width: nonzero_width,
            height: nonzero_height,
            frame_rate,
            time_base,
            range,
        };
        Ok(Self {
            input,
            stream: index,
            codec,
            decoded: ffmpeg::frame::Video::empty(),
            info,
            unreadable: 0,
        })
    }

    pub fn stream(&self) -> &StreamInfo {
        &self.info
    }

    pub fn unreadable_packets_after_last_frame(&self) -> u32 {
        self.unreadable
    }

    pub fn next(&mut self, into: &mut YuvFrame) -> Result<Option<FrameInfo>, Error> {
        loop {
            match self.codec.receive_frame(&mut self.decoded) {
                Ok(()) => return self.deliver(into).map(Some),
                Err(ffmpeg::Error::Eof) => return Ok(None),
                Err(ffmpeg::Error::InvalidData) => self.unreadable = self.unreadable.saturating_add(1),
                Err(ffmpeg::Error::Other { errno }) if errno == EAGAIN => self.feed()?,
                Err(source) => return Err(Error::Decode(LibraryError::new(source))),
            }
        }
    }

    fn feed(&mut self) -> Result<(), Error> {
        loop {
            let mut packet = ffmpeg::Packet::empty();
            match packet.read(&mut self.input) {
                Ok(()) if packet.stream() == self.stream => return self.send(&packet),
                Ok(()) => {}
                Err(ffmpeg::Error::Eof) => {
                    return self.codec.send_eof().map_err(LibraryError::new).map_err(Error::Decode);
                }
                Err(source) => return Err(Error::Read(LibraryError::new(source))),
            }
        }
    }

    fn send(&mut self, packet: &ffmpeg::Packet) -> Result<(), Error> {
        match self.codec.send_packet(packet) {
            Err(ffmpeg::Error::InvalidData) => {
                self.unreadable = self.unreadable.saturating_add(1);
                Ok(())
            }
            sent => sent.map_err(LibraryError::new).map_err(Error::Decode),
        }
    }

    fn deliver(&mut self, into: &mut YuvFrame) -> Result<FrameInfo, Error> {
        let decoded = &self.decoded;
        let range = color_range(decoded.format(), decoded.color_range())?;
        let (width, height) = (self.info.width(), self.info.height());
        same_size((width, height), (decoded.width(), decoded.height()))?;
        into.reshape(width, height, range).map_err(Error::Size)?;
        unpack(
            [0, 1, 2].map(|index| (decoded.data(index), decoded.stride(index))),
            into,
        )?;
        Ok(FrameInfo {
            key: decoded.is_key(),
            unreadable_packets: mem::take(&mut self.unreadable),
        })
    }
}

fn frame_rate(average: ffmpeg::Rational, real: ffmpeg::Rational) -> Result<Rational, Error> {
    positive(average)
        .or_else(|| positive(real))
        .ok_or(Error::UnknownFrameRate)
}

fn positive(rate: ffmpeg::Rational) -> Option<Rational> {
    Rational::new(rate.numerator(), rate.denominator())
        .ok()
        .filter(|rate| rate.num() > 0)
}

fn color_range(format: Pixel, range: Range) -> Result<ColorRange, Error> {
    match (format, range) {
        (Pixel::YUV420P, Range::MPEG) => Ok(ColorRange::Limited),
        (Pixel::YUV420P, Range::JPEG) | (Pixel::YUVJ420P, _) => Ok(ColorRange::Full),
        (Pixel::YUV420P, Range::Unspecified) => Ok(ColorRange::Unspecified),
        (other, _) => Err(Error::UnsupportedPixelFormat {
            format: other.descriptor().map_or("unknown", |descriptor| descriptor.name()),
        }),
    }
}

fn same_size((width, height): (u32, u32), (found_width, found_height): (u32, u32)) -> Result<(), Error> {
    let same = (width, height) == (found_width, found_height);
    same.then_some(()).ok_or(Error::SizeChanged {
        width,
        height,
        found_width,
        found_height,
    })
}

fn unpack(sources: [(&[u8], usize); 3], into: &mut YuvFrame) -> Result<(), Error> {
    let copied = sources
        .into_iter()
        .zip(into.planes_mut())
        .all(|((source, stride), (plane, row))| {
            let rows = plane.len() / row;
            planes::copy_rows(source, stride, plane, row, row, rows)
        });
    copied.then_some(()).ok_or(Error::TruncatedFrame)
}
