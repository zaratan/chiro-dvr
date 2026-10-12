use draw::canvas::{self, Canvas};
use draw::color::INK;
use media::convert::{self, Converter, SourceRange};
use media::frame::{self, ColorRange, YuvFrame};
use types::frame::FrameIndex;

use crate::overlay::Overlay;

#[cfg(test)]
mod tests;

const DISPLAY: SourceRange = SourceRange::Full;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot convert the frame for drawing")]
    Convert(#[source] convert::Error),
    #[error("cannot hold the frame for drawing")]
    Canvas(#[source] canvas::Error),
    #[error("cannot hold the drawn frame")]
    Frame(#[source] frame::Error),
}

#[derive(Debug)]
pub(crate) struct Picture {
    converter: Converter,
    canvas: Canvas,
    drawn: YuvFrame,
}

impl Picture {
    pub(crate) fn new(width: u32, height: u32) -> Result<Self, Error> {
        Ok(Self {
            converter: Converter::new(width, height, DISPLAY).map_err(Error::Convert)?,
            canvas: Canvas::new(width, height, INK).map_err(Error::Canvas)?,
            drawn: YuvFrame::new(width, height, ColorRange::Limited).map_err(Error::Frame)?,
        })
    }

    pub(crate) fn draw(&mut self, source: &YuvFrame, overlay: &Overlay, frame: FrameIndex) -> Result<&YuvFrame, Error> {
        self.converter
            .yuv_to_rgba(source, self.canvas.rgba_mut())
            .map_err(Error::Convert)?;
        overlay.draw(&mut self.canvas, frame);
        self.converter
            .rgba_to_yuv(self.canvas.rgba(), &mut self.drawn)
            .map_err(Error::Convert)?;
        Ok(&self.drawn)
    }
}
