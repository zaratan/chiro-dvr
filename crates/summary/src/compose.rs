use draw::canvas::{self, Canvas, Outline, PixelBox};
use draw::color::{INK, PALETTE, PANEL, SEPARATOR};
use draw::style::SummaryStyle;
use draw::text::Typeface;

use crate::geometry::{distance, drawn};
use crate::header::Labels;
use crate::layout::PageLayout;
use crate::legend::{LegendEntry, legend_panel};
use crate::marker::{Marker, draw_marker};

#[cfg(test)]
mod tests;

const LEADER_HALO: f32 = 2.0;

pub fn backdrop(mut background: Canvas, masks: &[PixelBox]) -> Canvas {
    for mask in masks {
        background.fill_rectangle(*mask, PANEL);
    }
    background.dim();
    background
}

#[derive(Debug, Clone, Copy)]
pub struct Pen<'a> {
    pub typeface: &'a Typeface,
    pub style: &'a SummaryStyle,
    pub labels: &'a dyn Labels,
}

pub fn compose(backdrop: &Canvas, page: &PageLayout<'_>, pen: Pen<'_>) -> Result<Canvas, canvas::Error> {
    let style = pen.style;
    let mut image = backdrop.clone();
    let color = |index: usize| PALETTE[index % PALETTE.len()];
    let lines: Vec<Vec<_>> = page
        .marks
        .iter()
        .map(|mark| mark.path.points().iter().copied().map(drawn).collect())
        .collect();
    for line in &lines {
        image.polyline(line, INK, style.outline);
    }
    for (line, mark) in lines.iter().zip(&page.marks) {
        image.polyline(line, color(mark.color), style.line);
    }
    let ring = Outline {
        color: INK,
        width: style.border,
    };
    for mark in &page.marks {
        if let Some(head) = mark.head {
            image.polygon(&head.map(drawn), Some(color(mark.color)), Some(ring));
        }
    }
    let entries: Vec<LegendEntry> = page
        .marks
        .iter()
        .map(|mark| LegendEntry {
            label: mark.track.id().to_string(),
            color: color(mark.color),
            text: mark.start.clone(),
        })
        .collect();
    let detached: Vec<_> = page
        .marks
        .iter()
        .filter(|mark| distance(mark.anchor, mark.center) > f64::from(style.radius))
        .map(|mark| ([drawn(mark.anchor), drawn(mark.center)], color(mark.color)))
        .collect();
    for (leader, _) in &detached {
        image.polyline(leader, INK, style.border + LEADER_HALO);
    }
    for (leader, leader_color) in &detached {
        image.polyline(leader, *leader_color, style.border);
    }
    for (entry, mark) in entries.iter().zip(&page.marks) {
        let marker = Marker {
            center: mark.center,
            label: &entry.label,
            color: entry.color,
        };
        draw_marker(&mut image, pen.typeface, marker, style);
    }
    let header: Vec<String> = page.header.iter().map(|line| pen.labels.line(line)).collect();
    let mut panel = legend_panel(pen.typeface, &header, &entries, image.height(), style)?;
    let separator = PixelBox {
        left: 0,
        top: 0,
        right: style.border as i32 - 1,
        bottom: panel.height() as i32 - 1,
    };
    panel.fill_rectangle(separator, SEPARATOR);
    let mut page_canvas = Canvas::new(image.width() + panel.width(), image.height(), PANEL)?;
    page_canvas.paste(&image, 0, 0);
    page_canvas.paste(&panel, image.width() as i32, 0);
    Ok(page_canvas)
}
