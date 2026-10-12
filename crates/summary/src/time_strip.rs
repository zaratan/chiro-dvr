use draw::canvas::{self, Canvas, PixelBox, Point};
use draw::color::{PANEL, PANEL_TEXT, Rgb, SEPARATOR};
use draw::style::SummaryStyle;
use draw::text::Typeface;

use crate::header::{Band, Labels, clock};
use crate::periods::TimeRange;

#[cfg(test)]
mod tests;

pub(crate) const NOT_ANALYSED: Rgb = Rgb::new(150, 150, 150);
pub(crate) const UNREADABLE: Rgb = PANEL_TEXT;
pub(crate) const OTHER_START: Rgb = Rgb::new(110, 110, 110);
const MIN_SPAN_PX: f32 = 2.0;
const MINUTE_STEPS: [u32; 4] = [1, 2, 5, 10];
const SECONDS_PER_MINUTE: f64 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StartTick {
    pub seconds: f64,
    pub color: Rgb,
}

#[derive(Debug, Clone, Copy)]
pub struct Strip<'a> {
    pub window: TimeRange,
    pub ignored: &'a [TimeRange],
    pub damaged: &'a [TimeRange],
    pub starts: &'a [StartTick],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StripGeometry {
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) top: f32,
    pub(crate) band: f32,
    pub(crate) gap: f32,
    digit: f32,
    margin: f32,
}

impl StripGeometry {
    pub(crate) fn new(width: u32, typeface: &Typeface, style: &SummaryStyle, labels: &dyn Labels) -> Self {
        let widest_label = [Band::NotAnalysed, Band::Unreadable]
            .map(|band| typeface.measure(&labels.band(band), style.text))
            .into_iter()
            .fold(0.0, f32::max);
        Self {
            left: 2.0 * style.margin + widest_label,
            right: width as f32 - style.margin,
            top: style.margin,
            band: style.text.digit_px().round(),
            gap: (style.margin / 2.0).round(),
            digit: style.text.digit_px(),
            margin: style.margin,
        }
    }

    pub(crate) fn band_top(&self, row: usize) -> f32 {
        self.top + row as f32 * (self.band + self.gap)
    }

    pub(crate) fn axis(&self) -> f32 {
        self.band_top(2) + self.gap
    }

    fn tick_baseline(&self) -> f32 {
        self.axis() + 2.0 * self.gap + self.digit
    }

    pub(crate) fn height(&self) -> f32 {
        self.tick_baseline() + self.margin
    }

    pub(crate) fn x(&self, window: TimeRange, seconds: f64) -> f32 {
        let length = window[1] - window[0];
        if length.is_nan() || length <= 0.0 {
            return self.left;
        }
        let share = ((seconds - window[0]) / length).clamp(0.0, 1.0) as f32;
        self.left + share * (self.right - self.left)
    }

    pub(crate) fn span(&self, window: TimeRange, range: TimeRange) -> (f32, f32) {
        let to = self
            .x(window, range[1])
            .max(self.x(window, range[0]) + MIN_SPAN_PX)
            .min(self.right);
        let from = self.x(window, range[0]).min(to - MIN_SPAN_PX);
        (from, to)
    }
}

pub(crate) fn minute_step(window: TimeRange, geometry: &StripGeometry, room: f32) -> u32 {
    let [.., widest] = MINUTE_STEPS;
    let minutes = ((window[1] - window[0]) / SECONDS_PER_MINUTE).max(f64::MIN_POSITIVE);
    let per_minute = f64::from(geometry.right - geometry.left) / minutes;
    MINUTE_STEPS
        .into_iter()
        .find(|&step| per_minute * f64::from(step) >= f64::from(room))
        .unwrap_or(widest)
}

pub fn draw_strip(
    strip: Strip<'_>,
    width: u32,
    typeface: &Typeface,
    style: &SummaryStyle,
    labels: &dyn Labels,
) -> Result<Canvas, canvas::Error> {
    let geometry = StripGeometry::new(width, typeface, style, labels);
    let mut canvas = Canvas::new(width, geometry.height().ceil() as u32, PANEL)?;
    let rows = [
        (Band::NotAnalysed, strip.ignored, NOT_ANALYSED),
        (Band::Unreadable, strip.damaged, UNREADABLE),
    ];
    for (row, (band, ranges, color)) in rows.into_iter().enumerate() {
        let top = geometry.band_top(row);
        let baseline = Point {
            x: style.margin,
            y: top + f32::midpoint(geometry.band, style.text.digit_px()),
        };
        canvas.text(typeface, &labels.band(band), baseline, style.text, PANEL_TEXT);
        canvas.rectangle(
            pixels(geometry.left, top, geometry.right, top + geometry.band),
            SEPARATOR,
            1,
        );
        for &range in ranges {
            let (from, to) = geometry.span(strip.window, range);
            canvas.fill_rectangle(pixels(from, top, to, top + geometry.band), color);
        }
    }
    let axis = geometry.axis();
    for tick in strip.starts {
        let x = geometry.x(strip.window, tick.seconds);
        canvas.line(
            Point {
                x,
                y: axis - geometry.gap,
            },
            Point { x, y: axis },
            tick.color,
            style.border,
        );
    }
    canvas.line(
        Point {
            x: geometry.left,
            y: axis,
        },
        Point {
            x: geometry.right,
            y: axis,
        },
        SEPARATOR,
        1.0,
    );
    let room = typeface.measure(&clock(strip.window[1]), style.text) + style.margin;
    let step = minute_step(strip.window, &geometry, room);
    let first = (strip.window[0] / SECONDS_PER_MINUTE).ceil() as u32;
    let last = (strip.window[1] / SECONDS_PER_MINUTE).floor() as u32;
    for minute in (first.div_ceil(step) * step..=last).step_by(step as usize) {
        let seconds = f64::from(minute) * SECONDS_PER_MINUTE;
        let x = geometry.x(strip.window, seconds);
        canvas.line(
            Point { x, y: axis },
            Point {
                x,
                y: axis + geometry.gap,
            },
            PANEL_TEXT,
            1.0,
        );
        let text = clock(seconds);
        let half = typeface.measure(&text, style.text) / 2.0;
        let baseline = Point {
            x: (x - half).clamp(0.0, (width as f32 - 2.0 * half).max(0.0)),
            y: geometry.tick_baseline(),
        };
        canvas.text(typeface, &text, baseline, style.text, PANEL_TEXT);
    }
    Ok(canvas)
}

fn pixels(left: f32, top: f32, right: f32, bottom: f32) -> PixelBox {
    PixelBox {
        left: left.floor() as i32,
        top: top.floor() as i32,
        right: (right.ceil() as i32).saturating_sub(1),
        bottom: (bottom.ceil() as i32).saturating_sub(1),
    }
}
