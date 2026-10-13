use draw::canvas::{self, Canvas};
use draw::color::INK;
use media::convert::{self, Converter, Region, SourceRange};
use media::frame::{self, ColorRange, YuvFrame};
use types::frame::FrameIndex;

#[cfg(test)]
mod tests;

const DISPLAY: SourceRange = SourceRange::FullBlackAt16;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot convert the frame for drawing")]
    Convert(#[source] convert::Error),
    #[error("cannot hold the frame for drawing")]
    Canvas(#[source] canvas::Error),
    #[error("cannot hold the drawn frame")]
    Frame(#[source] frame::Error),
}

pub(crate) trait Marks {
    fn zones(&self, frame: FrameIndex, width: u32, height: u32) -> Vec<Region>;
    fn draw(&self, canvas: &mut Canvas, frame: FrameIndex);
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

    pub(crate) fn draw(
        &mut self,
        source: &YuvFrame,
        marks: &impl Marks,
        frame: FrameIndex,
    ) -> Result<&YuvFrame, Error> {
        self.converter
            .display(source, &mut self.drawn)
            .map_err(Error::Convert)?;
        let zones = marks.zones(frame, source.width(), source.height());
        for zone in &zones {
            self.converter
                .yuv_to_rgba_region(source, self.canvas.rgba_mut(), *zone)
                .map_err(Error::Convert)?;
        }
        marks.draw(&mut self.canvas, frame);
        for zone in &zones {
            self.converter
                .rgba_to_yuv_region(self.canvas.rgba(), &mut self.drawn, *zone)
                .map_err(Error::Convert)?;
        }
        Ok(&self.drawn)
    }
}
