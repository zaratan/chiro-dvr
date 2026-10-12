pub mod arrows;
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
use draw::style::style_for;
use draw::text::{self, Typeface};
use files::names::Part;

use crate::compose::{Pen, backdrop, compose};
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
}

pub fn pages(
    summary: &Summary<'_>,
    background: Canvas,
    masks: &[PixelBox],
    labels: &dyn Labels,
) -> Result<Vec<Page>, Error> {
    let typeface = Typeface::roboto().map_err(Error::Font)?;
    let widest_label = summary
        .tracks
        .iter()
        .map(types::track::Track::id)
        .max()
        .unwrap_or_default()
        .to_string();
    let width = NonZeroU32::new(background.width()).unwrap_or(NonZeroU32::MIN);
    let style = style_for(width, &widest_label, &typeface);
    let frame = Frame {
        width: f64::from(background.width()),
        height: f64::from(background.height()),
    };
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
                minutes: page.minutes,
                part: page.part,
            })
        })
        .collect()
}
