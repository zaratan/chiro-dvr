use std::ffi::CStr;
use std::fmt;
use std::fs::File;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::ptr;

use ffmpeg::format::Pixel;
use ffmpeg::util::color::{Primaries, Range, Space, TransferCharacteristic};
use ffmpeg::util::error::EAGAIN;
use ffmpeg_next as ffmpeg;
use tempfile::TempPath;
use types::rational::Rational;

use crate::failure::LibraryError;
use crate::frame::{ColorRange, YuvFrame};
use crate::{init, path, planes};

pub mod fallback;
pub mod fanout;
#[cfg(test)]
mod tests;

const MP4: &CStr = c"mp4";
const VIDEOTOOLBOX_QUALITY: i32 = 65;
const OPENH264_QP: i32 = 23;
const OPENH264_FOOTPRINT: u64 = 17_000_000;
const VIDEOTOOLBOX_FOOTPRINT: u64 = 180_000_000;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("H.264 needs an even size, not {width}×{height}")]
    Size { width: u32, height: u32 },
    #[error("{frame_rate} is not a frame rate")]
    FrameRate { frame_rate: Rational },
    #[error("this FFmpeg has no {name} encoder")]
    EncoderMissing { name: &'static str },
    #[error("cannot write {}: FFmpeg cannot take this path", path.display())]
    UnusablePath { path: PathBuf },
    #[error("cannot create {}", path.display())]
    Create {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot start the {name} encoder")]
    Open {
        name: &'static str,
        #[source]
        source: LibraryError,
    },
    #[error("a {found_width}×{found_height} frame pushed to a {width}×{height} encoder")]
    FrameSize {
        width: u32,
        height: u32,
        found_width: u32,
        found_height: u32,
    },
    #[error("the encoder writes limited-range video, not a full-range frame")]
    FullRange,
    #[error("the encoder frame cannot hold the pushed planes")]
    FrameLayout,
    #[error("the encoder failed")]
    Encode(#[source] LibraryError),
    #[error("cannot open {} for writing", path.display())]
    OpenFile {
        path: PathBuf,
        #[source]
        source: LibraryError,
    },
    #[error("{} does not start on a key frame", path.display())]
    NoKeyFrame { path: PathBuf },
    #[error("cannot write {}", path.display())]
    Writing {
        path: PathBuf,
        #[source]
        source: LibraryError,
    },
    #[error("{} would hold no frame", path.display())]
    EmptyClip { path: PathBuf },
    #[error("{} ended before its last frame", path.display())]
    ShortClip { path: PathBuf },
    #[error("the encoder gave a packet without a timestamp")]
    UntimedPacket,
    #[error("cannot move the finished video to {}", path.display())]
    Persist {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::UnusablePath { path }
            | Self::Create { path, .. }
            | Self::OpenFile { path, .. }
            | Self::NoKeyFrame { path }
            | Self::Writing { path, .. }
            | Self::EmptyClip { path }
            | Self::ShortClip { path }
            | Self::Persist { path, .. } => Some(path),
            _ => None,
        }
    }

    pub fn concerns_the_destination(&self) -> bool {
        matches!(
            self,
            Self::UnusablePath { .. } | Self::Create { .. } | Self::Persist { .. } | Self::OpenFile { .. }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    OpenH264,
    VideoToolbox,
}

impl Codec {
    /// Bytes one open 1440×1080 encoder holds, measured on the extract, outside-process service included.
    pub fn footprint(self) -> u64 {
        match self {
            Self::OpenH264 => OPENH264_FOOTPRINT,
            Self::VideoToolbox => VIDEOTOOLBOX_FOOTPRINT,
        }
    }

    fn encoder_name(self) -> &'static str {
        match self {
            Self::OpenH264 => "libopenh264",
            Self::VideoToolbox => "h264_videotoolbox",
        }
    }

    fn tune(self, context: &mut ffmpeg::encoder::video::Video) -> (ffmpeg::codec::Flags, ffmpeg::Dictionary<'static>) {
        let mut options = ffmpeg::Dictionary::new();
        match self {
            Self::OpenH264 => {
                context.set_qmin(OPENH264_QP);
                context.set_qmax(OPENH264_QP);
                options.set("profile", "high");
                options.set("rc_mode", "quality");
                (ffmpeg::codec::Flags::empty(), options)
            }
            Self::VideoToolbox => {
                context.set_global_quality(VIDEOTOOLBOX_QUALITY * ffmpeg::ffi::FF_QP2LAMBDA);
                options.set("allow_sw", "0");
                (ffmpeg::codec::Flags::QSCALE, options)
            }
        }
    }
}

impl fmt::Display for Codec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::OpenH264 => "openh264",
            Self::VideoToolbox => "VideoToolbox hardware",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    width: NonZeroU32,
    height: NonZeroU32,
    frame_rate: Rational,
    codec: Codec,
    key_interval: Option<NonZeroU32>,
}

impl Settings {
    pub fn new(width: u32, height: u32, frame_rate: Rational, codec: Codec) -> Result<Self, Error> {
        let size = NonZeroU32::new(width).zip(NonZeroU32::new(height));
        let Some((width, height)) = size.filter(|(width, height)| width.get() % 2 == 0 && height.get() % 2 == 0) else {
            return Err(Error::Size { width, height });
        };
        if frame_rate.num() <= 0 {
            return Err(Error::FrameRate { frame_rate });
        }
        Ok(Self {
            width,
            height,
            frame_rate,
            codec,
            key_interval: None,
        })
    }

    /// Without it, openh264 writes a single key frame.
    #[must_use]
    pub fn with_key_interval(self, frames: NonZeroU32) -> Self {
        Self {
            key_interval: Some(frames),
            ..self
        }
    }

    pub(crate) fn with_codec(self, codec: Codec) -> Self {
        Self { codec, ..self }
    }
}

/// Writes an H.264 mp4 under a temporary name in the destination folder; only `finish` gives it its name.
pub struct Encoder {
    context: ffmpeg::encoder::Video,
    frame: ffmpeg::frame::Video,
    codec_time_base: ffmpeg::Rational,
    next_pts: i64,
    destination: Destination,
    codec: Codec,
}

impl fmt::Debug for Encoder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Encoder")
            .field("destination", &self.destination.path)
            .field("codec", &self.codec)
            .finish_non_exhaustive()
    }
}

impl Encoder {
    pub fn create(path: &Path, settings: &Settings) -> Result<Self, Error> {
        let opened = Opened::new(settings)?;
        let destination = Destination::create(path, &opened)?;
        Ok(Self {
            context: opened.context,
            frame: opened.frame,
            codec_time_base: opened.codec_time_base,
            next_pts: 0,
            destination,
            codec: settings.codec,
        })
    }

    pub fn codec(&self) -> Codec {
        self.codec
    }

    pub fn push(&mut self, frame: &YuvFrame) -> Result<(), Error> {
        fill(&mut self.frame, frame)?;
        self.frame.set_pts(Some(self.next_pts));
        self.next_pts += 1;
        send(&mut self.context, Some(&self.frame))?;
        self.drain()
    }

    pub fn finish(mut self) -> Result<(), Error> {
        send(&mut self.context, None)?;
        self.drain()?;
        self.destination.close()
    }

    fn drain(&mut self) -> Result<(), Error> {
        while let Some(packet) = receive(&mut self.context)? {
            self.destination.write(packet, self.codec_time_base, 0)?;
        }
        Ok(())
    }
}

struct Opened {
    context: ffmpeg::encoder::Video,
    frame: ffmpeg::frame::Video,
    codec_time_base: ffmpeg::Rational,
    codec: ffmpeg::Codec,
}

impl Opened {
    fn new(settings: &Settings) -> Result<Self, Error> {
        init::ffmpeg();
        let kind = settings.codec;
        let name = kind.encoder_name();
        let codec = ffmpeg::encoder::find_by_name(name).ok_or(Error::EncoderMissing { name })?;
        let failed = |source| Error::Open {
            name,
            source: LibraryError::new(source),
        };
        let rate = settings.frame_rate;
        let codec_time_base = ffmpeg::Rational::new(rate.den(), rate.num());
        let mut context = ffmpeg::codec::context::Context::new_with_codec(codec)
            .encoder()
            .video()
            .map_err(failed)?;
        context.set_width(settings.width.get());
        context.set_height(settings.height.get());
        context.set_format(Pixel::YUV420P);
        context.set_color_range(Range::MPEG);
        context.set_colorspace(Space::BT709);
        context.set_color_primaries(Primaries::BT709);
        context.set_color_transfer_characteristic(TransferCharacteristic::BT709);
        context.set_time_base(codec_time_base);
        context.set_frame_rate(Some(ffmpeg::Rational::new(rate.num(), rate.den())));
        if let Some(frames) = settings.key_interval {
            context.set_gop(frames.get());
        }
        context.set_max_b_frames(0);
        let (flags, options) = kind.tune(&mut context);
        context.set_flags(flags | ffmpeg::codec::Flags::GLOBAL_HEADER);
        let context = context.open_with(options).map_err(failed)?;
        let frame = ffmpeg::frame::Video::new(Pixel::YUV420P, settings.width.get(), settings.height.get());
        Ok(Self {
            context,
            frame,
            codec_time_base,
            codec,
        })
    }
}

struct Destination {
    output: ffmpeg::format::context::Output,
    stream: usize,
    stream_time_base: ffmpeg::Rational,
    temporary: TempPath,
    path: PathBuf,
}

impl Destination {
    fn create(path: &Path, opened: &Opened) -> Result<Self, Error> {
        let unusable = || Error::UnusablePath {
            path: path.to_path_buf(),
        };
        if path::location(path).is_none() {
            return Err(unusable());
        }
        let partial = path::partial(path).ok_or_else(unusable)?;
        let create = |source| Error::Create {
            path: path.to_path_buf(),
            source,
        };
        let temporary = TempPath::try_from_path(&partial).map_err(create)?;
        File::create(&temporary).map_err(create)?;
        let location = path::location(&temporary).ok_or_else(unusable)?;
        let writing = |source| Error::Writing {
            path: path.to_path_buf(),
            source: LibraryError::new(source),
        };
        let mut output = mp4_output(&location).map_err(|source| Error::OpenFile {
            path: path.to_path_buf(),
            source: LibraryError::new(source),
        })?;
        let mut stream = output.add_stream(opened.codec).map_err(writing)?;
        stream.set_parameters(&opened.context);
        stream.set_time_base(opened.codec_time_base);
        let index = stream.index();
        output.write_header().map_err(writing)?;
        let stream_time_base = output
            .stream(index)
            .map_or(opened.codec_time_base, |stream| stream.time_base());
        Ok(Self {
            output,
            stream: index,
            stream_time_base,
            temporary,
            path: path.to_path_buf(),
        })
    }

    fn write(
        &mut self,
        mut packet: ffmpeg::Packet,
        codec_time_base: ffmpeg::Rational,
        first: i64,
    ) -> Result<(), Error> {
        packet.set_stream(self.stream);
        packet.set_pts(packet.pts().map(|pts| pts - first));
        packet.set_dts(packet.dts().map(|dts| dts - first));
        if packet.duration() == 0 {
            packet.set_duration(1);
        }
        packet.rescale_ts(codec_time_base, self.stream_time_base);
        packet
            .write_interleaved(&mut self.output)
            .map_err(|source| Error::Writing {
                path: self.path.clone(),
                source: LibraryError::new(source),
            })
    }

    fn close(mut self) -> Result<(), Error> {
        self.output.write_trailer().map_err(|source| Error::Writing {
            path: self.path.clone(),
            source: LibraryError::new(source),
        })?;
        let Self {
            output,
            temporary,
            path,
            ..
        } = self;
        drop(output);
        temporary.persist(&path).map_err(|failure| Error::Persist {
            path,
            source: failure.error,
        })?;
        Ok(())
    }
}

fn fill(frame: &mut ffmpeg::frame::Video, source: &YuvFrame) -> Result<(), Error> {
    let (width, height) = (frame.width(), frame.height());
    let (found_width, found_height) = (source.width(), source.height());
    if (found_width, found_height) != (width, height) {
        return Err(Error::FrameSize {
            width,
            height,
            found_width,
            found_height,
        });
    }
    if source.range() == ColorRange::Full {
        return Err(Error::FullRange);
    }
    make_writable(frame)?;
    for (index, (plane, row)) in source.planes().into_iter().enumerate() {
        let stride = frame.stride(index);
        if !planes::copy_rows(plane, row, frame.data_mut(index), stride, row, plane.len() / row) {
            return Err(Error::FrameLayout);
        }
    }
    Ok(())
}

fn make_writable(frame: &mut ffmpeg::frame::Video) -> Result<(), Error> {
    // SAFETY: the frame owns its AVFrame, and the exclusive borrow keeps everything else away from it.
    let pointer = unsafe { frame.as_mut_ptr() };
    // SAFETY: `pointer` is a live AVFrame from `frame::Video::new`; the call may replace its buffers, and the exclusive borrow keeps every slice of the old ones out of scope.
    let status = unsafe { ffmpeg::ffi::av_frame_make_writable(pointer) };
    if status < 0 {
        return Err(Error::Encode(LibraryError::new(ffmpeg::Error::from(status))));
    }
    Ok(())
}

fn send(context: &mut ffmpeg::encoder::Video, frame: Option<&ffmpeg::frame::Video>) -> Result<(), Error> {
    let sent = match frame {
        Some(frame) => context.send_frame(frame),
        None => context.send_eof(),
    };
    sent.map_err(LibraryError::new).map_err(Error::Encode)
}

fn receive(context: &mut ffmpeg::encoder::Video) -> Result<Option<ffmpeg::Packet>, Error> {
    let mut packet = ffmpeg::Packet::empty();
    match context.receive_packet(&mut packet) {
        Ok(()) => Ok(Some(packet)),
        Err(ffmpeg::Error::Eof) => Ok(None),
        Err(ffmpeg::Error::Other { errno }) if errno == EAGAIN => Ok(None),
        Err(source) => Err(Error::Encode(LibraryError::new(source))),
    }
}

fn mp4_output(location: &CStr) -> Result<ffmpeg::format::context::Output, ffmpeg::Error> {
    let mut context = ptr::null_mut();
    // SAFETY: `context` is a valid out pointer, and both strings are NUL-terminated and outlive the call.
    let status = unsafe {
        ffmpeg::ffi::avformat_alloc_output_context2(&raw mut context, ptr::null(), MP4.as_ptr(), location.as_ptr())
    };
    if status < 0 {
        return Err(ffmpeg::Error::from(status));
    }
    // SAFETY: on success `context` is a fresh AVFormatContext owned by nothing else; `Output` frees it and its `pb`.
    let mut output = unsafe { ffmpeg::format::context::Output::wrap(context) };
    // SAFETY: `output` owns its live context, and nothing else borrows it here.
    let pointer = unsafe { output.as_mut_ptr() };
    // SAFETY: `pointer` is the live context above; its `pb` field is a plain pointer slot.
    let io = unsafe { &raw mut (*pointer).pb };
    // SAFETY: `io` is the still-null `pb` of the live context, and `location` is NUL-terminated.
    let status = unsafe { ffmpeg::ffi::avio_open(io, location.as_ptr(), ffmpeg::ffi::AVIO_FLAG_WRITE) };
    if status < 0 {
        return Err(ffmpeg::Error::from(status));
    }
    Ok(output)
}
