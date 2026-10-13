use std::path::Path;

use media::encode::{self, Codec};
use media::frame::YuvFrame;
use types::frame::FrameIndex;

use crate::ahead::serve_ahead;
use crate::clips::Error;
use crate::encoders::{Encoders, Open, Sink};
use crate::overlay::Overlay;
use crate::picture::Picture;
use crate::schedule::Hold;
use crate::source::{Origin, Source};
use crate::windows::Group;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct Rendering<'a, O> {
    pub(crate) source: &'a Source,
    pub(crate) overlay: &'a Overlay,
    pub(crate) encoders: &'a Encoders<'a, O>,
    pub(crate) paths: &'a [Option<&'a Path>],
}

#[derive(Debug)]
pub(crate) struct Written {
    pub(crate) origin: Option<Origin>,
    pub(crate) failed: Option<encode::Error>,
}

impl<O: Open> Rendering<'_, O> {
    pub(crate) fn render(
        &self,
        group: &Group,
        picture: &mut Option<Picture>,
        hold: &impl Hold,
    ) -> Result<Written, Error> {
        let Some(first) = group.members().first() else {
            return Ok(Written {
                origin: None,
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
            let drawn = drawn(picture, pixels, self.overlay, frame)?;
            let (sink, codec) = match &mut writer {
                Some(writer) => writer,
                None => writer.insert(self.encoders.open(lead, drawn.width(), drawn.height())?),
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
        if let Some(cause) = failed {
            return Ok(Written {
                origin: None,
                failed: Some(cause),
            });
        }
        let origin = served?;
        let finished = writer.map_or(Ok(None), |(sink, codec)| match sink.finish() {
            Ok(()) => Ok(None),
            Err(cause) => self.failed(lead, codec, cause).map(Some),
        });
        Ok(Written {
            origin: Some(origin),
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
