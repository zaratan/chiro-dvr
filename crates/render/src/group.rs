use std::path::Path;

use draw::color::Rgb;
use media::encode::{self, Codec, Stamps};
use media::frame::YuvFrame;
use types::frame::FrameIndex;
use types::track::Track;

use crate::ahead::serve_ahead;
use crate::clips::Error;
use crate::encoders::{Encoders, Open, Purpose, Sink};
use crate::ink::{Ink, ink_at};
use crate::overlay::Overlay;
use crate::picture::{Marks, Picture};
use crate::schedule::Hold;
use crate::source::{Origin, Source};
use crate::windows::{Group, Window};
use crate::work::{Paths, Work};
use crate::zoom::{Magnifier, ZoomTrail, crop};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct Rendering<'a, O> {
    pub(crate) source: &'a Source,
    pub(crate) overlay: &'a Overlay,
    pub(crate) encoders: &'a Encoders<'a, O>,
    pub(crate) paths: Paths<'a>,
    pub(crate) tracks: &'a [Track],
    pub(crate) inks: &'a [Ink],
    pub(crate) trail_frames: usize,
}

#[derive(Debug)]
pub(crate) struct Written {
    pub(crate) restarts: usize,
    pub(crate) failed: Option<encode::Error>,
}

#[derive(Debug, Default)]
pub(crate) struct Scratch {
    picture: Option<Picture>,
    zoomed: Option<YuvFrame>,
}

#[derive(Debug)]
struct View {
    magnifier: Magnifier,
    trail: ZoomTrail,
}

impl<O: Open> Rendering<'_, O> {
    pub(crate) fn render(&self, work: &Work, scratch: &mut Scratch, hold: &impl Hold) -> Result<Written, Error> {
        match work {
            Work::Annotated(path) => self.annotated(path, scratch, hold),
            Work::Clips(group) => self.clips(group, scratch, hold),
            Work::Zoom(window) => self.zoom(*window, scratch, hold),
        }
    }

    fn clips(&self, group: &Group, scratch: &mut Scratch, hold: &impl Hold) -> Result<Written, Error> {
        let Some(first) = group.members().first() else {
            return Ok(Written {
                restarts: 0,
                failed: None,
            });
        };
        let lead = self.path(first.track)?;
        let mut writer: Option<(O::Sink, Codec)> = None;
        let mut pending = group.members().iter().peekable();
        let mut failed = None;
        let served = serve_ahead(self.source, group.span(), |frame, pixels| {
            if hold.stopped() {
                return Err(Error::Stopped);
            }
            let drawn = drawn(&mut scratch.picture, pixels, self.overlay, frame)?;
            let (sink, codec) = match &mut writer {
                Some(writer) => writer,
                None => writer.insert(
                    self.encoders
                        .open(lead, drawn.width(), drawn.height(), Purpose::Clips)?,
                ),
            };
            while let Some(member) = pending.next_if(|member| member.span.first() == frame) {
                let path = self.path(member.track)?;
                let frames = member.span.frame_count();
                if let Err(cause) = sink.open(path, frames) {
                    failed = Some(self.failed(path, *codec, cause)?);
                    return Err(Error::Stopped);
                }
            }
            if let Err(cause) = sink.push(drawn) {
                failed = Some(self.failed(lead, *codec, cause)?);
                return Err(Error::Stopped);
            }
            Ok(())
        });
        self.written(served.map(|origin| restarts(&origin)), failed, writer, lead)
    }

    fn annotated(&self, path: &Path, scratch: &mut Scratch, hold: &impl Hold) -> Result<Written, Error> {
        let pieces = self.source.pieces();
        let stamps =
            Stamps::new(&self.source.stamps(&pieces), self.source.time_base()).map_err(|source| Error::Stamps {
                path: path.to_path_buf(),
                source,
            })?;
        let mut writer: Option<(O::Sink, Codec)> = None;
        let mut failed = None;
        let mut served = Ok(0);
        for piece in &pieces {
            let read = serve_ahead(self.source, *piece, |frame, pixels| {
                if hold.stopped() {
                    return Err(Error::Stopped);
                }
                let drawn = drawn(&mut scratch.picture, pixels, self.overlay, frame)?;
                let (sink, codec) = match &mut writer {
                    Some(writer) => writer,
                    None => match self.start(path, (drawn.width(), drawn.height()), Purpose::Annotated, |sink| {
                        sink.open_stamped(path, stamps.clone())
                    })? {
                        Ok(started) => writer.insert(started),
                        Err(cause) => {
                            failed = Some(cause);
                            return Err(Error::Stopped);
                        }
                    },
                };
                if let Err(cause) = sink.push(drawn) {
                    failed = Some(self.failed(path, *codec, cause)?);
                    return Err(Error::Stopped);
                }
                Ok(())
            });
            served = served.and_then(|count| read.map(|origin| count + restarts(&origin)));
            if served.is_err() {
                break;
            }
        }
        self.written(served, failed, writer, path)
    }

    fn zoom(&self, window: Window, scratch: &mut Scratch, hold: &impl Hold) -> Result<Written, Error> {
        let path = self.zoom_path(window.track)?;
        let track = self
            .tracks
            .get(window.track)
            .ok_or(Error::UnknownTrack { track: window.track })?;
        let mut writer: Option<(O::Sink, Codec, View)> = None;
        let mut failed = None;
        let served = serve_ahead(self.source, window.span, |frame, pixels| {
            if hold.stopped() {
                return Err(Error::Stopped);
            }
            let (sink, codec, view) = match &mut writer {
                Some(writer) => writer,
                None => match self.start(path, (pixels.width(), pixels.height()), Purpose::Zoom, |sink| {
                    sink.open(path, window.span.frame_count())
                })? {
                    Ok((sink, codec)) => writer.insert((
                        sink,
                        codec,
                        self.view(track, ink_at(self.inks, window.track).trail, pixels),
                    )),
                    Err(cause) => {
                        failed = Some(cause);
                        return Err(Error::Stopped);
                    }
                },
            };
            let zoomed = view.magnifier.magnify(pixels, &mut scratch.zoomed)?;
            let drawn = drawn(&mut scratch.picture, zoomed, &view.trail, frame)?;
            if let Err(cause) = sink.push(drawn) {
                failed = Some(self.failed(path, *codec, cause)?);
                return Err(Error::Stopped);
            }
            Ok(())
        });
        let writer = writer.map(|(sink, codec, _)| (sink, codec));
        self.written(served.map(|origin| restarts(&origin)), failed, writer, path)
    }

    fn view(&self, track: &Track, color: Rgb, pixels: &YuvFrame) -> View {
        let (width, height) = (pixels.width(), pixels.height());
        let crop = crop(track, width, height);
        View {
            magnifier: Magnifier::new(crop, width, height),
            trail: ZoomTrail::new(track, color, crop, (width, height), self.trail_frames),
        }
    }

    fn start(
        &self,
        path: &Path,
        (width, height): (u32, u32),
        purpose: Purpose,
        open_output: impl FnOnce(&mut O::Sink) -> Result<(), encode::Error>,
    ) -> Result<Result<(O::Sink, Codec), encode::Error>, Error> {
        let (mut sink, codec) = self.encoders.open(path, width, height, purpose)?;
        match open_output(&mut sink) {
            Ok(()) => Ok(Ok((sink, codec))),
            Err(cause) => self.failed(path, codec, cause).map(Err),
        }
    }

    fn written(
        &self,
        served: Result<usize, Error>,
        failed: Option<encode::Error>,
        writer: Option<(O::Sink, Codec)>,
        lead: &Path,
    ) -> Result<Written, Error> {
        if let Some(cause) = failed {
            return Ok(Written {
                restarts: 0,
                failed: Some(cause),
            });
        }
        let restarts = served?;
        let finished = writer.map_or(Ok(None), |(sink, codec)| match sink.finish() {
            Ok(()) => Ok(None),
            Err(cause) => self.failed(lead, codec, cause).map(Some),
        });
        Ok(Written {
            restarts,
            failed: finished?,
        })
    }

    fn failed(&self, lead: &Path, codec: Codec, cause: encode::Error) -> Result<encode::Error, Error> {
        if codec != Codec::VideoToolbox || cause.concerns_the_destination() {
            let path = cause.path().unwrap_or(lead).to_path_buf();
            return Err(Error::writing(&path, cause));
        }
        self.encoders.set_hardware_aside();
        Ok(cause)
    }

    fn path(&self, track: usize) -> Result<&Path, Error> {
        self.paths
            .clips
            .get(track)
            .copied()
            .flatten()
            .ok_or(Error::UnknownTrack { track })
    }

    fn zoom_path(&self, track: usize) -> Result<&Path, Error> {
        self.paths
            .zooms
            .get(track)
            .copied()
            .flatten()
            .ok_or(Error::UnknownTrack { track })
    }
}

fn restarts(origin: &Origin) -> usize {
    usize::from(matches!(origin, Origin::Start(_)))
}

fn drawn<'a>(
    picture: &'a mut Option<Picture>,
    pixels: &YuvFrame,
    marks: &impl Marks,
    frame: FrameIndex,
) -> Result<&'a YuvFrame, Error> {
    let picture = match picture {
        Some(picture) => picture,
        None => picture.insert(Picture::new(pixels.width(), pixels.height()).map_err(Error::Picture)?),
    };
    picture.draw(pixels, marks, frame).map_err(Error::Picture)
}
