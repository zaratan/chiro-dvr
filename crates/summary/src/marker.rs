use draw::canvas::{Canvas, Outline, Point};
use draw::color::{INK, Rgb};
use draw::style::SummaryStyle;
use draw::text::Typeface;

use crate::geometry::{Spot, drawn};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Marker<'a> {
    pub center: Spot,
    pub label: &'a str,
    pub color: Rgb,
}

pub fn draw_marker(canvas: &mut Canvas, typeface: &Typeface, marker: Marker<'_>, style: &SummaryStyle) {
    let center = drawn(marker.center);
    let ring = Outline {
        color: INK,
        width: style.border,
    };
    canvas.disk(center, style.radius, marker.color, Some(ring));
    let width = typeface.measure(marker.label, style.label);
    let baseline = Point {
        x: center.x - width / 2.0,
        y: center.y + style.label.digit_px() / 2.0,
    };
    canvas.text(typeface, marker.label, baseline, style.label, INK);
}
