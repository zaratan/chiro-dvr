use std::num::{NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};

use media::decode::{self, ActiveThreads, Decoder, FrameInfo, Threads};
use media::frame::{self, ColorRange, YuvFrame};
use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::size::SourceSize;

use crate::lost::{Losses, losses};
use crate::read_ahead::{Stop, read_ahead};

#[cfg(test)]
mod tests;

pub(crate) const READ_AHEAD: NonZeroUsize = NonZeroUsize::MIN.saturating_add(3);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot decode {}", path.display())]
    Decode {
        path: PathBuf,
        #[source]
        source: decode::Error,
    },
    #[error("{} announces no frame", path.display())]
    NoAnnouncedFrames { path: PathBuf },
    #[error(
        "{}: decoded {decoded} frames, expected {announced} announced − {} lost before the first − {} lost at \
         timestamp jumps − {} unreadable after the last",
        path.display(),
        losses.before_first,
        losses.at_jumps,
        losses.after_last
    )]
    FrameCount {
        path: PathBuf,
        announced: usize,
        losses: Losses,
        decoded: usize,
    },
    #[error("cannot hold the frames of {}", path.display())]
    Frame {
        path: PathBuf,
        #[source]
        source: frame::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Video {
    path: PathBuf,
    size: SourceSize,
    frame_rate: FrameRate,
    time_base: Rational,
    start: Option<Pts>,
    announced_frames: NonZeroUsize,
}

impl Video {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> SourceSize {
        self.size
    }

    pub fn frame_rate(&self) -> FrameRate {
        self.frame_rate
    }

    pub fn time_base(&self) -> Rational {
        self.time_base
    }

    pub fn start(&self) -> Option<Pts> {
        self.start
    }

    pub fn announced_frames(&self) -> NonZeroUsize {
        self.announced_frames
    }
}

#[derive(Debug)]
pub struct Input {
    video: Video,
    decoder: Decoder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    video: Video,
    frames: Vec<FrameInfo>,
    losses: Losses,
}

impl Reading {
    pub fn video(&self) -> &Video {
        &self.video
    }

    pub fn frames(&self) -> &[FrameInfo] {
        &self.frames
    }

    pub fn losses(&self) -> Losses {
        self.losses
    }

    pub fn key_frames(&self) -> Vec<FrameIndex> {
        self.frames
            .iter()
            .filter(|info| info.is_key())
            .map(|info| info.index())
            .collect()
    }
}

impl Input {
    pub fn open(path: &Path, threads: Threads) -> Result<Self, Error> {
        let decoder = Decoder::open(path, threads).map_err(|source| Error::Decode {
            path: path.to_path_buf(),
            source,
        })?;
        let stream = *decoder.stream();
        let video = Video {
            path: path.to_path_buf(),
            size: stream.size(),
            frame_rate: stream.frame_rate(),
            time_base: stream.time_base(),
            start: stream.start(),
            announced_frames: announced(path, stream.announced_frames())?,
        };
        Ok(Self { video, decoder })
    }

    #[must_use]
    pub fn with_seam(self, seam: Box<dyn media::decode::PacketSeam>) -> Self {
        Self {
            decoder: self.decoder.with_seam(seam),
            ..self
        }
    }

    pub fn video(&self) -> &Video {
        &self.video
    }

    pub fn active_threads(&self) -> ActiveThreads {
        self.decoder.active_threads()
    }

    pub fn read<E: From<Error>>(self, visit: impl FnMut(FrameInfo, &YuvFrame) -> Result<(), E>) -> Result<Reading, E> {
        self.read_ahead_by(READ_AHEAD, visit)
    }

    pub(crate) fn read_ahead_by<E: From<Error>>(
        self,
        ahead: NonZeroUsize,
        mut visit: impl FnMut(FrameInfo, &YuvFrame) -> Result<(), E>,
    ) -> Result<Reading, E> {
        let Self { video, mut decoder } = self;
        let path = video.path.as_path();
        let size = video.size;
        let template =
            YuvFrame::new(size.width().get(), size.height().get(), ColorRange::Limited).map_err(|source| {
                Error::Frame {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
        let mut frames = Vec::new();
        let outcome = read_ahead(
            ahead,
            || template.clone(),
            |slot| next_frame(&mut decoder, slot, path),
            |info, frame| {
                frames.push(info);
                visit(info, frame)
            },
        );
        match outcome {
            Ok(()) => {}
            Err(Stop::Source(error)) => return Err(error.into()),
            Err(Stop::Consumer(error)) => return Err(error),
        }
        let pts: Vec<Option<Pts>> = frames.iter().map(|info| info.pts()).collect();
        let after_last = usize::try_from(decoder.unreadable_packets_after_last_frame()).unwrap_or(usize::MAX);
        let losses = explained(&video, &pts, after_last)?;
        Ok(Reading { video, frames, losses })
    }
}

fn next_frame(decoder: &mut Decoder, slot: &mut YuvFrame, path: &Path) -> Result<Option<FrameInfo>, Error> {
    decoder.next(slot).map_err(|source| Error::Decode {
        path: path.to_path_buf(),
        source,
    })
}

fn announced(path: &Path, frames: Option<NonZeroU64>) -> Result<NonZeroUsize, Error> {
    frames
        .and_then(|frames| usize::try_from(frames.get()).ok())
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| Error::NoAnnouncedFrames {
            path: path.to_path_buf(),
        })
}

fn explained(video: &Video, pts: &[Option<Pts>], after_last: usize) -> Result<Losses, Error> {
    let losses = losses(pts, video.start, after_last);
    let announced = video.announced_frames.get();
    if announced.checked_sub(losses.total()) == Some(pts.len()) {
        Ok(losses)
    } else {
        Err(Error::FrameCount {
            path: video.path.clone(),
            announced,
            losses,
            decoded: pts.len(),
        })
    }
}
