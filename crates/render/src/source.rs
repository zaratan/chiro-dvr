use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use media::decode::placed::{Placed, SeekError};
use media::decode::{self, Decoder, Threads};
use media::frame::{self, ColorRange, YuvFrame};
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::span::Span;
use types::timeline::Timeline;

use self::numbering::Numbering;

mod numbering;
mod resume;
#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot read {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: decode::Error,
    },
    #[error("cannot hold a decoded frame")]
    Frame(#[source] frame::Error),
    #[error("frame {frame} has a pts that does not come after the one of the frame before")]
    UnorderedPts { frame: FrameIndex },
    #[error("frames {first} to {last} reach past the {frames} frames of the video")]
    OutsideVideo {
        first: FrameIndex,
        last: FrameIndex,
        frames: usize,
    },
    #[error("frame {frame} has no pts, so it cannot be found again")]
    NoPts { frame: FrameIndex },
    #[error("the decoder handed back a frame at pts {pts}, which the timeline does not know")]
    UnknownPts { pts: i64 },
    #[error("frame {frame} came back after frame {after}")]
    OutOfOrder { frame: FrameIndex, after: FrameIndex },
    #[error("frame {frame} was not served")]
    Missing { frame: FrameIndex },
}

#[derive(Debug)]
pub enum Origin {
    Key(FrameIndex),
    Start(Restart),
}

#[derive(Debug)]
pub enum Restart {
    NoCleanKey,
    Unplaced(SeekError),
    Overshot { landing: FrameIndex, first: FrameIndex },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Walk {
    Served,
    Overshot(FrameIndex),
}

/// Serves frames chosen by their pts, with the pixels a single-threaded reading gives from the start.
#[derive(Debug)]
pub struct Source {
    path: PathBuf,
    frames: usize,
    rate: FrameRate,
    numbering: Numbering,
    keys: Vec<FrameIndex>,
    errors: Vec<FrameIndex>,
}

impl Source {
    pub fn new(path: &Path, timeline: &Timeline, keys: &[FrameIndex], errors: &[FrameIndex]) -> Result<Self, Error> {
        let sorted = |frames: &[FrameIndex]| {
            let mut frames = frames.to_vec();
            frames.sort_unstable();
            frames.dedup();
            frames
        };
        Ok(Self {
            path: path.to_path_buf(),
            frames: timeline.frame_count(),
            rate: timeline.rate(),
            numbering: Numbering::of(timeline)?,
            keys: sorted(keys),
            errors: sorted(errors),
        })
    }

    pub fn frame_count(&self) -> usize {
        self.frames
    }

    pub fn rate(&self) -> FrameRate {
        self.rate
    }

    pub fn serve<E: From<Error>>(
        &self,
        span: Span,
        mut visit: impl FnMut(FrameIndex, &YuvFrame) -> Result<(), E>,
    ) -> Result<Origin, E> {
        self.check(span)?;
        let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).map_err(Error::Frame)?;
        let resume = resume::clean_key_before(span.first(), &self.keys, &self.errors, &self.numbering);
        let restart = match resume {
            None => Restart::NoCleanKey,
            Some((key, pts)) => match self.open(key, span.last())?.seek_before(pts) {
                Err(error) => Restart::Unplaced(error),
                Ok(placed) => match self.walk(placed, &mut frame, span, Some(key), &mut visit)? {
                    Walk::Served => return Ok(Origin::Key(key)),
                    Walk::Overshot(first) => Restart::Overshot { landing: key, first },
                },
            },
        };
        let start = FrameIndex::new(0);
        self.walk(
            self.open(start, span.last())?.at_start(),
            &mut frame,
            span,
            None,
            &mut visit,
        )?;
        Ok(Origin::Start(restart))
    }

    fn check(&self, span: Span) -> Result<(), Error> {
        if span.last().get() >= self.frames {
            return Err(Error::OutsideVideo {
                first: span.first(),
                last: span.last(),
                frames: self.frames,
            });
        }
        let unknown = (span.first().get()..=span.last().get())
            .map(FrameIndex::new)
            .find(|frame| self.numbering.pts(*frame).is_none());
        unknown.map_or(Ok(()), |frame| Err(Error::NoPts { frame }))
    }

    fn open(&self, from: FrameIndex, to: FrameIndex) -> Result<Decoder, Error> {
        Decoder::open(&self.path, self.threads(from, to)).map_err(|source| self.unreadable(source))
    }

    fn threads(&self, from: FrameIndex, to: FrameIndex) -> Threads {
        if self.errors.iter().any(|error| (from..=to).contains(error)) {
            Threads::Exactly(NonZeroUsize::MIN)
        } else {
            Threads::Automatic
        }
    }

    fn unreadable(&self, source: decode::Error) -> Error {
        Error::Read {
            path: self.path.clone(),
            source,
        }
    }

    fn walk<E: From<Error>>(
        &self,
        mut placed: Placed,
        frame: &mut YuvFrame,
        span: Span,
        landing: Option<FrameIndex>,
        visit: &mut impl FnMut(FrameIndex, &YuvFrame) -> Result<(), E>,
    ) -> Result<Walk, E> {
        let mut previous: Option<FrameIndex> = None;
        let mut wanted = span.first();
        while let Some(decoded) = placed.next(frame).map_err(|source| self.unreadable(source))? {
            let Some(pts) = decoded.pts() else {
                continue;
            };
            let index = self.numbering.frame(pts).ok_or(Error::UnknownPts { pts: pts.get() })?;
            match previous {
                None if landing.is_some_and(|landing| index > landing) => return Ok(Walk::Overshot(index)),
                Some(after) if index <= after => return Err(Error::OutOfOrder { frame: index, after }.into()),
                _ => previous = Some(index),
            }
            if index > wanted {
                return Err(Error::Missing { frame: wanted }.into());
            }
            if index == wanted {
                visit(index, frame)?;
                if index == span.last() {
                    return Ok(Walk::Served);
                }
                wanted = FrameIndex::new(index.get() + 1);
            }
        }
        Err(Error::Missing { frame: wanted }.into())
    }
}
