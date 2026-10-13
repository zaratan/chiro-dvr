use std::fmt;
use std::mem;
use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};

use ffmpeg::codec::packet::Mut as _;
use ffmpeg::codec::threading;
use ffmpeg::format::Pixel;
use ffmpeg::util::color::Range;
use ffmpeg::util::error::EAGAIN;
use ffmpeg_next as ffmpeg;
use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::size::SourceSize;

use crate::decode_errors::DecodeErrors;
use crate::failure::LibraryError;
use crate::frame::{self, ColorRange, YuvFrame};
use crate::{init, path, planes};

pub mod placed;
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
    #[error("the decoder handed back a frame stored bottom up, which is not handled")]
    FlippedFrame,
    #[error("the decoder handed back a frame shorter than its size")]
    TruncatedFrame,
    #[error("the video stream has an unusable frame size")]
    Size(#[source] frame::Error),
    #[error("cannot read the next packet")]
    Read(#[source] LibraryError),
    #[error("the decoder failed")]
    Decode(#[source] LibraryError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Threads {
    #[default]
    Automatic,
    Exactly(NonZeroUsize),
}

impl Threads {
    fn count(self) -> usize {
        match self {
            Self::Automatic => 0,
            Self::Exactly(count) => count.get(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveThreads {
    Single,
    Frame(NonZeroUsize),
    Slice(NonZeroUsize),
}

impl ActiveThreads {
    fn of(config: threading::Config) -> Self {
        match (config.kind, NonZeroUsize::new(config.count)) {
            (threading::Type::Frame, Some(count)) if count.get() > 1 => Self::Frame(count),
            (threading::Type::Slice, Some(count)) if count.get() > 1 => Self::Slice(count),
            _ => Self::Single,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamInfo {
    width: NonZeroU32,
    height: NonZeroU32,
    frame_rate: FrameRate,
    time_base: Rational,
    range: ColorRange,
    start: Option<Pts>,
    duration: Option<i64>,
    announced_frames: Option<NonZeroU64>,
}

impl StreamInfo {
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    pub fn height(&self) -> u32 {
        self.height.get()
    }

    pub fn size(&self) -> SourceSize {
        SourceSize::new(self.width, self.height)
    }

    pub fn frame_rate(&self) -> FrameRate {
        self.frame_rate
    }

    pub fn time_base(&self) -> Rational {
        self.time_base
    }

    pub fn range(&self) -> ColorRange {
        self.range
    }

    pub fn start(&self) -> Option<Pts> {
        self.start
    }

    /// The stream length the container announces, in the stream time base.
    pub fn duration(&self) -> Option<i64> {
        self.duration
    }

    pub fn announced_frames(&self) -> Option<NonZeroU64> {
        self.announced_frames
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameInfo {
    index: FrameIndex,
    pts: Option<Pts>,
    key: bool,
    decode_errors: DecodeErrors,
    unreadable_packets: u32,
}

impl FrameInfo {
    pub fn index(self) -> FrameIndex {
        self.index
    }

    pub fn pts(self) -> Option<Pts> {
        self.pts
    }

    pub fn is_key(self) -> bool {
        self.key
    }

    pub fn decode_errors(self) -> DecodeErrors {
        self.decode_errors
    }

    pub fn unreadable_packets(self) -> u32 {
        self.unreadable_packets
    }
}

pub struct Packet<'a> {
    packet: &'a mut ffmpeg::Packet,
    ordinal: usize,
}

impl Packet<'_> {
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub fn is_key(&self) -> bool {
        self.packet.is_key()
    }

    pub fn size(&self) -> usize {
        self.packet.size()
    }

    pub fn data_mut(&mut self) -> Option<&mut [u8]> {
        let pointer = self.packet.as_mut_ptr();
        // SAFETY: `pointer` is a live AVPacket; the call may copy its buffer, and `&mut self` keeps every slice of the old one out of scope.
        let status = unsafe { ffmpeg::ffi::av_packet_make_writable(pointer) };
        (status >= 0).then(|| self.packet.data_mut()).flatten()
    }

    pub fn shrink(&mut self, size: usize) {
        self.packet.shrink(size);
    }
}

impl fmt::Debug for Packet<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Packet")
            .field("ordinal", &self.ordinal)
            .field("key", &self.is_key())
            .field("size", &self.size())
            .finish()
    }
}

/// Sees each packet of the video stream, in reading order, before the decoder does.
pub trait PacketSeam: Send {
    fn pass(&mut self, packet: &mut Packet<'_>);
}

impl<F: FnMut(&mut Packet<'_>) + Send> PacketSeam for F {
    fn pass(&mut self, packet: &mut Packet<'_>) {
        self(packet);
    }
}

/// Unreadable packets are counted, not fatal; after an `Err` the decoder is unusable.
pub struct Decoder {
    input: ffmpeg::format::context::Input,
    stream: usize,
    codec: ffmpeg::decoder::Video,
    decoded: ffmpeg::frame::Video,
    info: StreamInfo,
    seam: Option<Box<dyn PacketSeam>>,
    packets: usize,
    delivered: usize,
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
    pub fn open(path: &Path, threads: Threads) -> Result<Self, Error> {
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
        let announced_frames = u64::try_from(stream.frames()).ok().and_then(NonZeroU64::new);
        let duration = Some(stream.duration()).filter(|duration| *duration > 0);
        let start = known_pts(stream.start_time());
        let mut context = ffmpeg::codec::context::Context::from_parameters(stream.parameters()).map_err(failed)?;
        let mut threading = threading::Config::kind(threading::Type::Frame);
        threading.count = threads.count();
        context.set_threading(threading);
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
            start,
            duration,
            announced_frames,
        };
        Ok(Self {
            input,
            stream: index,
            codec,
            decoded: ffmpeg::frame::Video::empty(),
            info,
            seam: None,
            packets: 0,
            delivered: 0,
            unreadable: 0,
        })
    }

    #[must_use]
    pub fn with_seam(mut self, seam: Box<dyn PacketSeam>) -> Self {
        self.seam = Some(seam);
        self
    }

    pub fn stream(&self) -> &StreamInfo {
        &self.info
    }

    pub fn active_threads(&self) -> ActiveThreads {
        ActiveThreads::of(self.codec.threading())
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
                Ok(()) if packet.stream() == self.stream => return self.send(&mut packet),
                Ok(()) => {}
                Err(ffmpeg::Error::Eof) => {
                    return self.codec.send_eof().map_err(LibraryError::new).map_err(Error::Decode);
                }
                Err(source) => return Err(Error::Read(LibraryError::new(source))),
            }
        }
    }

    fn send(&mut self, packet: &mut ffmpeg::Packet) -> Result<(), Error> {
        if let Some(seam) = self.seam.as_mut() {
            seam.pass(&mut Packet {
                packet,
                ordinal: self.packets,
            });
        }
        self.packets += 1;
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
        let strides = [0, 1, 2].map(|index| decoded.stride(index));
        upright(strides)?;
        unpack([0, 1, 2].map(|index| (decoded.data(index), strides[index])), into)?;
        let index = FrameIndex::new(self.delivered);
        self.delivered += 1;
        Ok(FrameInfo {
            index,
            pts: decoded.pts().map(Pts::new),
            key: decoded.is_key(),
            decode_errors: self.decode_errors(),
            unreadable_packets: mem::take(&mut self.unreadable),
        })
    }

    fn decode_errors(&self) -> DecodeErrors {
        // SAFETY: `decoded` owns its AVFrame for as long as `self`.
        let frame = unsafe { self.decoded.as_ptr() };
        // SAFETY: `frame` is the live AVFrame filled by the last `receive_frame`.
        let flags = unsafe { (*frame).decode_error_flags };
        DecodeErrors::from_bits(flags)
    }
}

fn known_pts(ticks: i64) -> Option<Pts> {
    (ticks != ffmpeg::ffi::AV_NOPTS_VALUE).then_some(Pts::new(ticks))
}

fn frame_rate(average: ffmpeg::Rational, real: ffmpeg::Rational) -> Result<FrameRate, Error> {
    positive(average)
        .or_else(|| positive(real))
        .and_then(|rate| FrameRate::new(rate).ok())
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

fn upright(strides: [usize; 3]) -> Result<(), Error> {
    let non_negative = strides.iter().all(|stride| isize::try_from(*stride).is_ok());
    non_negative.then_some(()).ok_or(Error::FlippedFrame)
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
