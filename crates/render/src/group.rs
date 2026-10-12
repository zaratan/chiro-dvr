use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use media::encode::{self, Codec};
use media::frame::YuvFrame;
use rayon::prelude::*;
use types::frame::FrameIndex;

use crate::ahead::serve_ahead;
use crate::clips::Error;
use crate::encoders::{Encoders, Open, Sink};
use crate::overlay::Overlay;
use crate::picture::Picture;
use crate::source::{Origin, Source};
use crate::windows::{Group, Member, Window};

#[derive(Debug)]
pub(crate) struct Rendering<'a, O> {
    pub(crate) source: &'a Source,
    pub(crate) overlay: &'a Overlay,
    pub(crate) encoders: &'a Encoders<'a, O>,
    pub(crate) paths: &'a [Option<&'a Path>],
}

#[derive(Debug)]
pub(crate) struct Redo {
    pub(crate) window: Window,
    pub(crate) path: PathBuf,
    pub(crate) cause: encode::Error,
}

#[derive(Debug)]
pub(crate) struct Written {
    pub(crate) origin: Origin,
    pub(crate) redo: Vec<Redo>,
}

#[derive(Debug)]
struct Writing<'a, S> {
    member: Member,
    path: &'a Path,
    codec: Codec,
    sink: S,
}

impl<O: Open> Rendering<'_, O> {
    pub(crate) fn render(
        &self,
        group: &Group,
        picture: &mut Option<Picture>,
        stop: &AtomicBool,
    ) -> Result<Written, Error> {
        let mut lanes: Vec<Option<Writing<'_, O::Sink>>> =
            std::iter::repeat_with(|| None).take(group.lanes()).collect();
        let mut pending = group.members().iter().peekable();
        let mut redo = Vec::new();
        let origin = serve_ahead(self.source, group.span(), |frame, pixels| {
            if stop.load(Ordering::Acquire) {
                return Err(Error::Stopped);
            }
            let drawn = drawn(picture, pixels, self.overlay, frame)?;
            while let Some(member) = pending.next_if(|member| member.window.span.first() == frame) {
                let path = self.path(member.window.track)?;
                let (sink, codec) = self.encoders.open(path, member.lane, drawn.width(), drawn.height())?;
                if let Some(lane) = lanes.get_mut(member.lane) {
                    *lane = Some(Writing {
                        member: *member,
                        path,
                        codec,
                        sink,
                    });
                }
            }
            let pushed: Vec<Result<(), encode::Error>> = lanes
                .par_iter_mut()
                .map(|lane| lane.as_mut().map_or(Ok(()), |writing| writing.sink.push(drawn)))
                .collect();
            for (lane, outcome) in lanes.iter_mut().zip(pushed) {
                let ends = lane
                    .as_ref()
                    .is_some_and(|writing| writing.member.window.span.last() == frame);
                if outcome.is_ok() && !ends {
                    continue;
                }
                let Some(writing) = lane.take() else {
                    continue;
                };
                let (member, path, codec) = (writing.member, writing.path, writing.codec);
                let written = outcome.and_then(|()| writing.sink.finish());
                if let Err(cause) = written {
                    redo.push(self.failed(member.window, path, codec, cause)?);
                }
            }
            Ok(())
        })?;
        Ok(Written { origin, redo })
    }

    fn failed(&self, window: Window, path: &Path, codec: Codec, cause: encode::Error) -> Result<Redo, Error> {
        if codec != Codec::VideoToolbox || cause.concerns_the_destination() {
            return Err(Error::writing(path, cause));
        }
        self.encoders.set_hardware_aside();
        Ok(Redo {
            window,
            path: path.to_path_buf(),
            cause,
        })
    }

    fn path(&self, track: usize) -> Result<&Path, Error> {
        self.paths
            .get(track)
            .copied()
            .flatten()
            .ok_or(Error::UnknownTrack { track })
    }
}

fn drawn<'a>(
    picture: &'a mut Option<Picture>,
    pixels: &YuvFrame,
    overlay: &Overlay,
    frame: FrameIndex,
) -> Result<&'a YuvFrame, Error> {
    let picture = match picture {
        Some(picture) => picture,
        None => picture.insert(Picture::new(pixels.width(), pixels.height()).map_err(Error::Picture)?),
    };
    picture.draw(pixels, overlay, frame).map_err(Error::Picture)
}
