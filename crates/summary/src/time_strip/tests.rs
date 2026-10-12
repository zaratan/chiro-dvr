use std::num::NonZeroU32;

use draw::canvas::Canvas;
use draw::color::{PALETTE, PANEL, Rgb, SEPARATOR};
use draw::style::{SummaryStyle, style_for};
use draw::text::Typeface;

use super::{NOT_ANALYSED, StartTick, Strip, StripGeometry, UNREADABLE, draw_strip, minute_step};
use crate::testing::DebugLabels;

const WIDTH: u32 = 1440;

fn tools() -> (Typeface, SummaryStyle) {
    let typeface = Typeface::roboto().unwrap();
    let style = style_for(NonZeroU32::new(WIDTH).unwrap(), "99", &typeface);
    (typeface, style)
}

fn geometry() -> StripGeometry {
    let (typeface, style) = tools();
    StripGeometry::new(WIDTH, &typeface, &style, &DebugLabels)
}

fn drawn(strip: Strip<'_>) -> Canvas {
    let (typeface, style) = tools();
    draw_strip(strip, WIDTH, &typeface, &style, &DebugLabels).unwrap()
}

fn at(canvas: &Canvas, x: u32, y: u32) -> Rgb {
    let index = ((y * canvas.width() + x) * 4) as usize;
    let pixel = &canvas.rgba()[index..index + 3];
    Rgb::new(pixel[0], pixel[1], pixel[2])
}

fn band_middle(row: usize) -> u32 {
    let strip = geometry();
    (strip.band_top(row) + strip.band / 2.0) as u32
}

fn middle_x(range: [f64; 2]) -> u32 {
    geometry().x(QUIET.window, f64::midpoint(range[0], range[1])) as u32
}

const QUIET: Strip<'static> = Strip {
    window: [0.0, 300.0],
    ignored: &[],
    damaged: &[],
    starts: &[],
};

#[test]
fn the_window_spans_the_strip_from_its_left_edge_to_its_right_edge() {
    let strip = geometry();
    assert_eq!(strip.x([0.0, 300.0], 0.0), strip.left);
    assert_eq!(strip.x([0.0, 300.0], 300.0), strip.right);
    assert!((strip.x([600.0, 900.0], 750.0) - f32::midpoint(strip.left, strip.right)).abs() < 1e-3);
    assert_eq!(strip.x([0.0, 300.0], 400.0), strip.right);
}

#[test]
fn a_one_second_span_on_a_ten_minute_strip_stays_at_least_two_pixels_wide() {
    let strip = geometry();
    let (from, to) = strip.span([0.0, 600.0], [10.0, 10.1]);
    assert!(to - from >= 2.0, "{from} {to}");
}

#[test]
fn a_short_span_at_the_end_of_the_window_stays_inside_the_band() {
    let strip = geometry();
    let (from, to) = strip.span([0.0, 300.0], [299.9, 300.0]);
    assert_eq!(to, strip.right);
    assert!(to - from >= 2.0, "{from} {to}");
}

#[test]
fn an_empty_or_undefined_window_puts_every_time_at_the_left_edge() {
    let strip = geometry();
    assert_eq!(strip.x([5.0, 5.0], 5.0), strip.left);
    assert_eq!(strip.x([0.0, f64::NAN], 5.0), strip.left);
}

#[test]
fn minutes_are_marked_one_by_one_when_there_is_room_and_more_sparsely_when_not() {
    let strip = geometry();
    assert_eq!(minute_step([0.0, 600.0], &strip, 80.0), 1);
    assert_eq!(minute_step([0.0, 600.0], &strip, 300.0), 5);
}

#[test]
fn each_kind_of_ignored_time_is_painted_in_its_own_band_and_tint() {
    let (ignored, damaged) = ([30.0, 60.0], [150.0, 180.0]);
    let canvas = drawn(Strip {
        ignored: &[ignored],
        damaged: &[damaged],
        ..QUIET
    });
    assert_eq!(at(&canvas, middle_x(ignored), band_middle(0)), NOT_ANALYSED);
    assert_eq!(at(&canvas, middle_x(ignored), band_middle(1)), PANEL);
    assert_eq!(at(&canvas, middle_x(damaged), band_middle(1)), UNREADABLE);
    assert_eq!(at(&canvas, middle_x(damaged), band_middle(0)), PANEL);
    assert_eq!(canvas.height(), geometry().height().ceil() as u32);
}

#[test]
fn an_empty_band_is_a_dark_frame_that_cannot_be_mistaken_for_a_full_one() {
    let canvas = drawn(QUIET);
    let strip = geometry();
    for row in 0..2 {
        let y = band_middle(row);
        let inside: Vec<Rgb> = (strip.left.ceil() as u32 + 1..strip.right as u32 - 1)
            .map(|x| at(&canvas, x, y))
            .collect();
        assert!(inside.iter().all(|&colour| colour == PANEL), "{row}");
        assert_eq!(at(&canvas, strip.left as u32, y), SEPARATOR);
    }
}

#[test]
fn every_minute_of_the_window_is_marked_on_the_axis_from_start_to_end() {
    let canvas = drawn(QUIET);
    let strip = geometry();
    let below_axis = (strip.axis() + strip.gap / 2.0) as u32;
    for minute in 0..=5 {
        let x = strip.x(QUIET.window, f64::from(minute) * 60.0).round() as u32;
        let marked = (x.saturating_sub(1)..=x + 1).any(|column| at(&canvas, column, below_axis) != PANEL);
        assert!(marked, "{minute}");
    }
    let between = strip.x(QUIET.window, 90.0) as u32;
    assert_eq!(at(&canvas, between, below_axis), PANEL);
}

#[test]
fn a_track_start_is_marked_on_the_time_axis_in_the_colour_of_its_track() {
    let starts = [StartTick {
        seconds: 120.0,
        color: PALETTE[4],
    }];
    let canvas = drawn(Strip {
        starts: &starts,
        ..QUIET
    });
    let strip = geometry();
    let x = strip.x(QUIET.window, 120.0).round() as u32;
    let above_axis = (strip.axis() - strip.gap / 2.0) as u32;
    assert!((x - 1..=x + 1).any(|column| at(&canvas, column, above_axis) == PALETTE[4]));
}

#[test]
fn eighty_seven_unreadable_spans_each_leave_a_mark_in_their_band() {
    let spans: Vec<[f64; 2]> = (0..87)
        .map(|k| [3.0 * f64::from(k) + 1.0, 3.0 * f64::from(k) + 2.0])
        .collect();
    let canvas = drawn(Strip {
        damaged: &spans,
        ..QUIET
    });
    for &range in &spans {
        assert_eq!(at(&canvas, middle_x(range), band_middle(1)), UNREADABLE, "{range:?}");
    }
}
