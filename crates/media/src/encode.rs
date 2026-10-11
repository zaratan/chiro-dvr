use std::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use ffmpeg::format::Pixel;
use ffmpeg::util::color::Range;
use ffmpeg::util::error::EAGAIN;
use ffmpeg_next as ffmpeg;
use tempfile::TempPath;
use types::rational::Rational;

use crate::failure::LibraryError;
use crate::frame::{ColorRange, YuvFrame};
use crate::{init, path, planes};

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("H.264 needs an even size, not {width}×{height}")]
    Size { width: u32, height: u32 },
    #[error("{frame_rate} is not a frame rate")]
    FrameRate { frame_rate: Rational },
    #[error("this FFmpeg has no {name} encoder")]
    EncoderMissing { name: &'static str },
    #[error("cannot write {}: the path is not UTF-8 or holds a NUL byte", path.display())]
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
    #[error("cannot write the video")]
    Write(#[source] LibraryError),
    #[error("cannot move the finished video to {}", path.display())]
    Persist {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    OpenH264,
    VideoToolbox { allow_software: bool },
}

impl Codec {
    fn encoder_name(self) -> &'static str {
        match self {
            Self::OpenH264 => "libopenh264",
            Self::VideoToolbox { .. } => "h264_videotoolbox",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    width: NonZeroU32,
    height: NonZeroU32,
    frame_rate: Rational,
    codec: Codec,
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
        })
    }
}

/// Writes an H.264 mp4 under a temporary name in the destination folder; only `finish` gives it its name.
pub struct Encoder {
    output: ffmpeg::format::context::Output,
    codec: ffmpeg::encoder::Video,
    frame: ffmpeg::frame::Video,
    stream: usize,
    codec_time_base: ffmpeg::Rational,
    stream_time_base: ffmpeg::Rational,
    next_pts: i64,
    temporary: TempPath,
    destination: PathBuf,
}

impl fmt::Debug for Encoder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Encoder")
            .field("destination", &self.destination)
            .finish_non_exhaustive()
    }
}

impl Encoder {
    pub fn create(path: &Path, settings: &Settings) -> Result<Self, Error> {
        init::ffmpeg();
        let name = settings.codec.encoder_name();
        let codec = ffmpeg::encoder::find_by_name(name).ok_or(Error::EncoderMissing { name })?;
        let unusable = || Error::UnusablePath {
            path: path.to_path_buf(),
        };
        path::for_ffmpeg(path).ok_or_else(unusable)?;
        let folder = path
            .parent()
            .filter(|folder| !folder.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let created = tempfile::Builder::new()
            .prefix(".encoding-")
            .suffix(".mp4")
            .tempfile_in(folder);
        let temporary = created
            .map_err(|source| Error::Create {
                path: path.to_path_buf(),
                source,
            })?
            .into_temp_path();
        let location = path::for_ffmpeg(&temporary).ok_or_else(unusable)?;
        let mut output = ffmpeg::format::output_as(location, "mp4")
            .map_err(LibraryError::new)
            .map_err(Error::Write)?;
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
        context.set_time_base(codec_time_base);
        context.set_frame_rate(Some(ffmpeg::Rational::new(rate.num(), rate.den())));
        if output.format().flags().contains(ffmpeg::format::Flags::GLOBAL_HEADER) {
            context.set_flags(ffmpeg::codec::Flags::GLOBAL_HEADER);
        }
        let mut options = ffmpeg::Dictionary::new();
        if let Codec::VideoToolbox { allow_software } = settings.codec {
            options.set("allow_sw", if allow_software { "1" } else { "0" });
        }
        let encoder = context.open_with(options).map_err(failed)?;
        let mut stream = output.add_stream(codec).map_err(failed)?;
        stream.set_parameters(&encoder);
        stream.set_time_base(codec_time_base);
        let index = stream.index();
        output.write_header().map_err(LibraryError::new).map_err(Error::Write)?;
        let stream_time_base = output
            .stream(index)
            .map_or(codec_time_base, |stream| stream.time_base());
        let frame = ffmpeg::frame::Video::new(Pixel::YUV420P, settings.width.get(), settings.height.get());
        Ok(Self {
            output,
            codec: encoder,
            frame,
            stream: index,
            codec_time_base,
            stream_time_base,
            next_pts: 0,
            temporary,
            destination: path.to_path_buf(),
        })
    }

    pub fn push(&mut self, frame: &YuvFrame) -> Result<(), Error> {
        let (width, height) = (self.frame.width(), self.frame.height());
        let (found_width, found_height) = (frame.width(), frame.height());
        if (found_width, found_height) != (width, height) {
            return Err(Error::FrameSize {
                width,
                height,
                found_width,
                found_height,
            });
        }
        if frame.range() == ColorRange::Full {
            return Err(Error::FullRange);
        }
        self.make_writable()?;
        for (index, (plane, row)) in frame.planes().into_iter().enumerate() {
            let stride = self.frame.stride(index);
            if !planes::copy_rows(plane, row, self.frame.data_mut(index), stride, row, plane.len() / row) {
                return Err(Error::FrameLayout);
            }
        }
        self.frame.set_pts(Some(self.next_pts));
        self.next_pts += 1;
        self.codec
            .send_frame(&self.frame)
            .map_err(LibraryError::new)
            .map_err(Error::Encode)?;
        self.drain()
    }

    pub fn finish(mut self) -> Result<(), Error> {
        self.codec
            .send_eof()
            .map_err(LibraryError::new)
            .map_err(Error::Encode)?;
        self.drain()?;
        self.output
            .write_trailer()
            .map_err(LibraryError::new)
            .map_err(Error::Write)?;
        let Self {
            output,
            temporary,
            destination,
            ..
        } = self;
        drop(output);
        let persisted = temporary.persist(&destination);
        persisted.map_err(|failure| Error::Persist {
            path: destination,
            source: failure.error,
        })?;
        Ok(())
    }

    fn make_writable(&mut self) -> Result<(), Error> {
        // SAFETY: the frame owns its AVFrame for as long as `self`, and nothing else borrows it here.
        let pointer = unsafe { self.frame.as_mut_ptr() };
        // SAFETY: `pointer` is a live AVFrame from `frame::Video::new`; the call may replace its buffers, and `&mut self` keeps every slice of the old ones out of scope.
        let status = unsafe { ffmpeg::ffi::av_frame_make_writable(pointer) };
        if status < 0 {
            return Err(Error::Encode(LibraryError::new(ffmpeg::Error::from(status))));
        }
        Ok(())
    }

    fn drain(&mut self) -> Result<(), Error> {
        loop {
            let mut packet = ffmpeg::Packet::empty();
            match self.codec.receive_packet(&mut packet) {
                Ok(()) => self.write(packet)?,
                Err(ffmpeg::Error::Eof) => return Ok(()),
                Err(ffmpeg::Error::Other { errno }) if errno == EAGAIN => return Ok(()),
                Err(source) => return Err(Error::Encode(LibraryError::new(source))),
            }
        }
    }

    fn write(&mut self, mut packet: ffmpeg::Packet) -> Result<(), Error> {
        packet.set_stream(self.stream);
        if packet.duration() == 0 {
            packet.set_duration(1);
        }
        packet.rescale_ts(self.codec_time_base, self.stream_time_base);
        packet
            .write_interleaved(&mut self.output)
            .map_err(LibraryError::new)
            .map_err(Error::Write)
    }
}
