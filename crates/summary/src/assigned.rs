use draw::color::Rgb;
use draw::text::Typeface;

use crate::compose::palette_color;
use crate::layout::{Summary, layout};
use crate::{Error, sized};

#[cfg(test)]
mod tests;

/// The colour each track of `summary` gets on its summary image of that size, in the order of its tracks.
pub fn track_colors(summary: &Summary<'_>, width: u32, height: u32) -> Result<Vec<Option<Rgb>>, Error> {
    let typeface = Typeface::roboto().map_err(Error::Font)?;
    let (style, frame) = sized(summary, width, height, &typeface);
    let pages = layout(summary, frame, &style);
    let color = |id: usize| {
        pages
            .iter()
            .flat_map(|page| &page.marks)
            .find(|mark| mark.track.id() == id)
            .map(|mark| palette_color(mark.color))
    };
    Ok(summary.tracks.iter().map(|track| color(track.id())).collect())
}
