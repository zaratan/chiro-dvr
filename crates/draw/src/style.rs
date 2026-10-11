use std::num::NonZeroU32;

use crate::text::{TextStyle, Typeface, Weight};

#[cfg(test)]
mod tests;

pub const TRACK_LABEL: TextStyle = TextStyle::trusted(17.0, Weight::SemiBold);
const MIN_RADIUS: f32 = 8.0;
const MIN_TEXT_PX: f32 = 8.0;
const MIN_ARROW: f32 = 8.0;
const MIN_MARGIN: f32 = 4.0;
const TEXT_TO_RADIUS: f32 = 0.9;
const WIDTH_PER_LINE: f32 = 480.0;
const WIDTH_PER_BORDER: f32 = 720.0;
const WIDTH_PER_RADIUS: f32 = 64.0;
const WIDTH_PER_OUTLINE: f32 = 960.0;
const WIDTH_PER_ARROW: f32 = 48.0;
const WIDTH_PER_MARGIN: f32 = 90.0;
const WIDTH_PER_NEIGHBOR: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SummaryStyle {
    pub line: f32,
    pub outline: f32,
    pub radius: f32,
    pub border: f32,
    pub arrow_length: f32,
    pub text: TextStyle,
    pub label: TextStyle,
    pub margin: f32,
    pub neighbor_distance: f32,
}

pub fn style_for(width: NonZeroU32, widest_label: &str, typeface: &Typeface) -> SummaryStyle {
    let width = pixels(width.get());
    let scaled = |per: f32, least: f32| (width / per).round().max(least);
    let line = scaled(WIDTH_PER_LINE, 1.0);
    let border = scaled(WIDTH_PER_BORDER, 1.0);
    let base_radius = scaled(WIDTH_PER_RADIUS, MIN_RADIUS);
    let digit_px = (base_radius * TEXT_TO_RADIUS).round().max(MIN_TEXT_PX);
    let text = TextStyle::trusted(digit_px, Weight::Regular);
    let label = TextStyle::trusted(digit_px, Weight::SemiBold);
    let label_width = typeface.measure(widest_label, label);
    SummaryStyle {
        line,
        outline: line + 2.0 * scaled(WIDTH_PER_OUTLINE, 1.0),
        radius: base_radius.max((label_width / 2.0).ceil() + 2.0 * border),
        border,
        arrow_length: scaled(WIDTH_PER_ARROW, MIN_ARROW),
        text,
        label,
        margin: scaled(WIDTH_PER_MARGIN, MIN_MARGIN),
        neighbor_distance: width / WIDTH_PER_NEIGHBOR,
    }
}

fn pixels(width: u32) -> f32 {
    width as f32
}
