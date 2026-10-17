pub mod arrows;
#[cfg(feature = "testing")]
pub mod assigned;
pub mod colors;
pub mod compose;
pub mod geometry;
pub mod header;
pub mod image_file;
pub mod layout;
pub mod legend;
pub mod marker;
pub mod periods;
pub mod placement;
#[cfg(test)]
mod testing;
pub mod time_strip;

use std::num::NonZeroU32;
use std::ops::Range;

use draw::canvas::{self, Canvas, PixelBox};
use draw::color::Rgb;
use draw::style::{SummaryStyle, style_for};
use draw::text::{self, Typeface};
use files::names::Part;

use crate::compose::{Pen, backdrop, compose, palette_color};
use crate::header::Labels;
use crate::layout::{Summary, layout};
use crate::placement::Frame;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the summary font cannot be read")]
    Font(#[source] text::Error),
    #[error("cannot draw the summary")]
    Canvas(#[source] canvas::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub minutes: Option<Range<u32>>,
    pub part: Option<Part>,
    pub canvas: Canvas,
    pub painted: Vec<Painted>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Painted {
    pub track: usize,
    pub color: Rgb,
}

pub fn pages(
    summary: &Summary<'_>,
    background: Canvas,
    masks: &[PixelBox],
    labels: &dyn Labels,
) -> Result<Vec<Page>, Error> {
    let typeface = Typeface::roboto().map_err(Error::Font)?;
    let (style, frame) = sized(summary, background.width(), background.height(), &typeface);
    let backdrop = backdrop(background, masks);
    let pen = Pen {
        typeface: &typeface,
        style: &style,
        labels,
    };
    layout(summary, frame, &style)
        .into_iter()
        .map(|page| {
            Ok(Page {
                canvas: compose(&backdrop, &page, pen).map_err(Error::Canvas)?,
                painted: page
                    .marks
                    .iter()
                    .map(|mark| Painted {
                        track: mark.track.id(),
                        color: palette_color(mark.color),
                    })
                    .collect(),
                minutes: page.minutes,
                part: page.part,
            })
        })
        .collect()
}

pub(crate) fn sized(summary: &Summary<'_>, width: u32, height: u32, typeface: &Typeface) -> (SummaryStyle, Frame) {
    let widest_label = summary
        .tracks
        .iter()
        .map(types::track::Track::id)
        .max()
        .unwrap_or_default()
        .to_string();
    let style = style_for(
        NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN),
        &widest_label,
        typeface,
    );
    let frame = Frame {
        width: f64::from(width),
        height: f64::from(height),
    };
    (style, frame)
}
