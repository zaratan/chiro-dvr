use draw::canvas::{self, Canvas, Outline, PixelBox};
use draw::color::{INK, PALETTE, PANEL, Rgb, SEPARATOR};
use draw::style::SummaryStyle;
use draw::text::Typeface;

use crate::geometry::drawn;
use crate::header::Labels;
use crate::layout::PageLayout;
use crate::legend::{LegendEntry, legend_panel};
use crate::marker::{Marker, draw_marker};
use crate::time_strip::{OTHER_START, StartTick, Strip, draw_strip};

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
    let lines: Vec<Vec<_>> = page
        .marks
        .iter()
        .map(|mark| mark.path.points().iter().copied().map(drawn).collect())
        .collect();
    for line in &lines {
        image.polyline(line, INK, style.outline);
    }
    for (line, mark) in lines.iter().zip(&page.marks) {
        image.polyline(line, palette_color(mark.color), style.line);
    }
    let ring = Outline {
        color: INK,
        width: style.border,
    };
    for mark in &page.marks {
        if let Some(head) = mark.head {
            image.polygon(&head.map(drawn), Some(palette_color(mark.color)), Some(ring));
        }
    }
    let entries: Vec<LegendEntry> = page
        .marks
        .iter()
        .map(|mark| LegendEntry {
            label: mark.track.id().to_string(),
            color: palette_color(mark.color),
            text: mark.start.clone(),
        })
        .collect();
    let detached: Vec<_> = page
        .marks
        .iter()
        .filter_map(|mark| {
            let leader = mark.leader(f64::from(style.radius))?;
            Some((leader.map(drawn), palette_color(mark.color)))
        })
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
    let panel = legend_panel(pen.typeface, &header, &entries, image.height(), style)?;
    let starts = start_ticks(page);
    let strip = Strip {
        window: page.window,
        ignored: &page.ignored,
        damaged: &page.damaged,
        starts: &starts,
    };
    let time_strip = draw_strip(strip, image.width(), pen.typeface, style, pen.labels)?;
    let height = image.height() + time_strip.height();
    let mut page_canvas = Canvas::new(image.width() + panel.width(), height, PANEL)?;
    page_canvas.paste(&image, 0, 0);
    page_canvas.paste(&time_strip, 0, image.height() as i32);
    page_canvas.paste(&panel, image.width() as i32, 0);
    let separator = PixelBox {
        left: image.width() as i32,
        top: 0,
        right: image.width() as i32 + style.border as i32 - 1,
        bottom: height as i32 - 1,
    };
    page_canvas.fill_rectangle(separator, SEPARATOR);
    Ok(page_canvas)
}

fn start_ticks(page: &PageLayout<'_>) -> Vec<StartTick> {
    let others = page.other_starts.iter().map(|&seconds| StartTick {
        seconds,
        color: OTHER_START,
    });
    let mine = page.marks.iter().map(|mark| StartTick {
        seconds: mark.start_seconds,
        color: palette_color(mark.color),
    });
    others.chain(mine).collect()
}

pub(crate) fn palette_color(index: usize) -> Rgb {
    PALETTE[index % PALETTE.len()]
}
