use std::error::Error as _;
use std::io;
use std::path::PathBuf;

use engine::detection;
use engine::input;
use media::decode;
use media::encode::fallback;
use media::failure::LibraryError;

use crate::options::Options;
use crate::process::Error;

#[cfg(test)]
mod tests;

pub const UNSUPPORTED: &str = "unsupported";
pub const ENCODING: &str = "encoding";
pub const UNEXPECTED: &str = "unexpected";
pub const TOO_MANY_TRACKS: &str = "too-many-tracks";
pub const INTERNAL: &str = "internal";
pub const REFERS_TO_DETAIL: [&str; 4] = [UNSUPPORTED, ENCODING, UNEXPECTED, INTERNAL];
const CAUSE_SEPARATOR: &str = ": ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cause {
    Missing,
    ReadRefused,
    NotAVideo,
    UnusableName,
    Damaged,
    Unsupported,
    Settings,
    DiskFull,
    Forbidden,
    InUse {
        file: PathBuf,
    },
    Encoding,
    TooManyTracks {
        count: usize,
        limit: usize,
        annotated: bool,
    },
    Unexpected,
}

impl Cause {
    pub const fn key(&self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::ReadRefused => "read-refused",
            Self::NotAVideo => "not-a-video",
            Self::UnusableName => "unusable-name",
            Self::Damaged => "damaged",
            Self::Unsupported => UNSUPPORTED,
            Self::Settings => "settings",
            Self::DiskFull => "disk-full",
            Self::Forbidden => "forbidden",
            Self::InUse { .. } => "in-use",
            Self::Encoding => ENCODING,
            Self::TooManyTracks { .. } => TOO_MANY_TRACKS,
            Self::Unexpected => UNEXPECTED,
        }
    }
}

pub fn chain(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        text.push_str(CAUSE_SEPARATOR);
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

pub fn cause(error: &Error, options: &Options) -> Cause {
    match error {
        Error::Detection(detection) => detecting(detection, options.tunes_the_frames()),
        Error::Folder { .. } | Error::Files(_) | Error::Overview(_) => writing(error).unwrap_or(Cause::Unexpected),
        Error::Video { source, .. } => match source.kind() {
            io::ErrorKind::NotFound => Cause::Missing,
            io::ErrorKind::PermissionDenied => Cause::ReadRefused,
            _ => Cause::Unexpected,
        },
        Error::Previous(previous) => in_use(previous).or_else(|| writing(error)).unwrap_or(Cause::Unexpected),
        Error::Encoders(fallback::Error::Scratch(_)) | Error::Source(_) => Cause::Encoding,
        Error::Encoders(_) | Error::Clips(_) => writing(error).unwrap_or(Cause::Encoding),
        Error::TooManyTracks {
            count,
            limit,
            annotated,
            ..
        } => Cause::TooManyTracks {
            count: *count,
            limit: *limit,
            annotated: *annotated,
        },
        Error::Progress(_) | Error::Timeline(_) | Error::Tracking(_) => Cause::Unexpected,
    }
}

fn detecting(error: &detection::Error, tuned: bool) -> Cause {
    match error {
        detection::Error::Input(input) => reading(input),
        detection::Error::Geometry { .. }
        | detection::Error::Blur { .. }
        | detection::Error::Mask { .. }
        | detection::Error::Config { .. } => {
            if tuned {
                Cause::Settings
            } else {
                Cause::Unsupported
            }
        }
    }
}

fn reading(error: &input::Error) -> Cause {
    match error {
        input::Error::Decode { source, .. } => decoding(source),
        input::Error::NoAnnouncedFrames { .. } | input::Error::FrameCount { .. } => Cause::Damaged,
        input::Error::Frame { .. } => Cause::Unexpected,
    }
}

fn decoding(error: &decode::Error) -> Cause {
    match error {
        decode::Error::UnusablePath { .. } => Cause::UnusableName,
        decode::Error::Open { source, .. } => match source.io_kind() {
            Some(io::ErrorKind::NotFound) => Cause::Missing,
            Some(io::ErrorKind::PermissionDenied) => Cause::ReadRefused,
            _ => Cause::NotAVideo,
        },
        decode::Error::NoVideoStream(_) => Cause::NotAVideo,
        decode::Error::NoDecoder { .. }
        | decode::Error::UnknownFrameRate
        | decode::Error::UnknownTimeBase
        | decode::Error::UnsupportedPixelFormat { .. }
        | decode::Error::SizeChanged { .. }
        | decode::Error::FlippedFrame
        | decode::Error::TruncatedFrame
        | decode::Error::Size(_) => Cause::Unsupported,
        decode::Error::Read(_) | decode::Error::Decode(_) => Cause::Damaged,
    }
}

fn writing(error: &Error) -> Option<Cause> {
    let mut link = error.source();
    while let Some(current) = link {
        let kind = current
            .downcast_ref::<io::Error>()
            .map(io::Error::kind)
            .or_else(|| current.downcast_ref::<LibraryError>().and_then(LibraryError::io_kind));
        match kind {
            Some(io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded) => return Some(Cause::DiskFull),
            Some(io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem) => {
                return Some(Cause::Forbidden);
            }
            _ => link = current.source(),
        }
    }
    None
}

#[cfg(windows)]
fn in_use(error: &files::error::Error) -> Option<Cause> {
    const SHARING_VIOLATION: i32 = 32;
    const LOCK_VIOLATION: i32 = 33;
    match error {
        files::error::Error::Remove { path, source }
        | files::error::Error::Read { path, source }
        | files::error::Error::SetAside { path, source }
            if matches!(source.raw_os_error(), Some(SHARING_VIOLATION | LOCK_VIOLATION)) =>
        {
            Some(Cause::InUse { file: path.clone() })
        }
        _ => None,
    }
}

#[cfg(not(windows))]
const fn in_use(_error: &files::error::Error) -> Option<Cause> {
    None
}
