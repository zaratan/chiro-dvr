use draw::canvas::{self, Canvas, Point};
use draw::color::{PANEL, PANEL_TEXT, Rgb};
use draw::style::SummaryStyle;
use draw::text::Typeface;

use crate::geometry::drawn;
use crate::marker::{Marker, draw_marker};

#[cfg(test)]
mod tests;

pub const ELLIPSIS: &str = "...";
pub const WIDEST_TIME: &str = "00:00";
const HEADER_LINE_SPACING: f32 = 1.8;
pub const MAX_HEADER_COLUMNS: f32 = 3.0;
const COLUMN_MARGINS: f32 = 3.0;

#[derive(Debug, Clone, PartialEq)]
pub struct LegendEntry {
    pub label: String,
    pub color: Rgb,
    pub text: String,
}

pub fn fit_text(typeface: &Typeface, text: &str, width: f32, style: &SummaryStyle) -> String {
    if typeface.measure(text, style.text) <= width {
        return text.to_owned();
    }
    let mut kept = text.to_owned();
    while !kept.is_empty() && typeface.measure(&format!("{kept}{ELLIPSIS}"), style.text) > width {
        kept.pop();
    }
    kept + ELLIPSIS
}

pub fn column_width(typeface: &Typeface, style: &SummaryStyle, texts: &[&str]) -> f32 {
    let widest = std::iter::once(WIDEST_TIME)
        .chain(texts.iter().copied())
        .map(|text| typeface.measure(text, style.text))
        .fold(0.0, f32::max);
    2.0 * style.radius + widest + COLUMN_MARGINS * style.margin
}

pub fn row_height(style: &SummaryStyle) -> f32 {
    2.0 * style.radius + style.margin / 2.0
}

pub fn header_line_height(style: &SummaryStyle) -> f32 {
    (style.text.digit_px() * HEADER_LINE_SPACING).round()
}

pub fn legend_width(
    typeface: &Typeface,
    header: &[String],
    entries_width: f32,
    column: f32,
    style: &SummaryStyle,
) -> u32 {
    let measured = |text: &str| typeface.measure(text, style.text) + 2.0 * style.margin;
    let name = header
        .first()
        .map_or(0.0, |name| measured(name).min(MAX_HEADER_COLUMNS * column));
    let others = header.iter().skip(1).map(|line| measured(line)).fold(0.0, f32::max);
    entries_width.max(name).max(others).ceil() as u32
}

pub fn legend_panel(
    typeface: &Typeface,
    header: &[String],
    entries: &[LegendEntry],
    height: u32,
    style: &SummaryStyle,
) -> Result<Canvas, canvas::Error> {
    let line_height = header_line_height(style);
    let top = style.margin + line_height * header.len() as f32 + style.margin;
    let rows = ((height as f32 - top - style.margin) / row_height(style))
        .floor()
        .max(1.0) as usize;
    let columns = entries.len().div_ceil(rows).max(1);
    let texts: Vec<&str> = entries.iter().map(|entry| entry.text.as_str()).collect();
    let column = column_width(typeface, style, &texts);
    let width = legend_width(typeface, header, columns as f32 * column + style.margin, column, style);
    let mut panel = Canvas::new(width, height, PANEL)?;
    let room = width as f32 - 2.0 * style.margin;
    for (index, text) in header.iter().enumerate() {
        let baseline = Point {
            x: style.margin,
            y: style.margin + line_height * (index + 1) as f32,
        };
        let line = fit_text(typeface, text, room, style);
        panel.text(typeface, &line, baseline, style.text, PANEL_TEXT);
    }
    for (index, entry) in entries.iter().enumerate() {
        let (column_index, row) = (index / rows, index % rows);
        let center = [
            f64::from(style.margin + column_index as f32 * column + style.radius),
            f64::from(top + row as f32 * row_height(style) + style.radius),
        ];
        let marker = Marker {
            center,
            label: &entry.label,
            color: entry.color,
        };
        draw_marker(&mut panel, typeface, marker, style);
        let baseline = Point {
            x: drawn(center).x + style.radius + style.margin,
            y: drawn(center).y + style.text.digit_px() / 2.0,
        };
        panel.text(typeface, &entry.text, baseline, style.text, PANEL_TEXT);
    }
    Ok(panel)
}
