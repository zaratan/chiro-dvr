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
use draw::color::Rgb;
use draw::style::style_for;
use draw::text::{self, Typeface};
use files::names::Part;

use crate::compose::{Pen, backdrop, compose, palette_color};
use crate::geometry::Spot;
use crate::header::Labels;
use crate::layout::{Mark, Summary, layout};
use crate::periods::TimeRange;
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
pub struct Summaries {
    pub backdrop: Canvas,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub minutes: Option<Range<u32>>,
    pub part: Option<Part>,
    pub canvas: Canvas,
    pub window: TimeRange,
    pub ignored: Vec<TimeRange>,
    pub damaged: Vec<TimeRange>,
    pub other_starts: Vec<f64>,
    pub radius: f64,
    pub painted: Vec<Painted>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Painted {
    pub track: usize,
    pub color: Rgb,
    pub path: Vec<Spot>,
    pub head: Option<[Spot; 3]>,
    pub dot: Spot,
    pub anchor: Option<Spot>,
}

impl Painted {
    fn of(mark: &Mark<'_>, radius: f64) -> Self {
        Self {
            track: mark.track.id(),
            color: palette_color(mark.color),
            path: mark.path.points().to_vec(),
            head: mark.head,
            dot: mark.center,
            anchor: mark.leader(radius).map(|[anchor, _]| anchor),
        }
    }
}

pub fn pages(
    summary: &Summary<'_>,
    background: Canvas,
    masks: &[PixelBox],
    labels: &dyn Labels,
) -> Result<Summaries, Error> {
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
    let radius = f64::from(style.radius);
    let pages = layout(summary, frame, &style)
        .into_iter()
        .map(|page| {
            let canvas = compose(&backdrop, &page, pen).map_err(Error::Canvas)?;
            Ok(Page {
                painted: page.marks.iter().map(|mark| Painted::of(mark, radius)).collect(),
                minutes: page.minutes,
                part: page.part,
                canvas,
                window: page.window,
                ignored: page.ignored,
                damaged: page.damaged,
                other_starts: page.other_starts,
                radius,
            })
        })
        .collect::<Result<_, Error>>()?;
    Ok(Summaries { backdrop, pages })
}
