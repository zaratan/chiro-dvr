use draw::canvas::{Canvas, PixelBox};
use draw::color::{PANEL, Rgb};

use super::backdrop;

const SCENE: Rgb = Rgb::new(200, 100, 50);

fn at(canvas: &Canvas, x: u32, y: u32) -> [u8; 3] {
    let index = ((y * canvas.width() + x) * 4) as usize;
    [canvas.rgba()[index], canvas.rgba()[index + 1], canvas.rgba()[index + 2]]
}

#[test]
fn display_masks_are_hidden_under_the_panel_colour_and_the_scene_is_kept_dimmed() {
    let display = PixelBox {
        left: 0,
        top: 0,
        right: 3,
        bottom: 1,
    };

    let dimmed = backdrop(Canvas::new(10, 6, SCENE).unwrap(), &[display]);

    assert_eq!(at(&dimmed, 3, 1), [19, 19, 19]);
    assert_eq!(at(&dimmed, 4, 1), [120, 60, 30]);
    assert_eq!(at(&dimmed, 0, 2), [120, 60, 30]);
    assert_eq!(PANEL, Rgb::new(32, 32, 32));
}

#[test]
fn nothing_is_hidden_without_display_masks() {
    let dimmed = backdrop(Canvas::new(10, 6, SCENE).unwrap(), &[]);

    assert!(
        dimmed
            .rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [120, 60, 30, 255])
    );
}

fn gray_page(tracks: &[types::track::Track]) -> Canvas {
    use std::num::NonZeroU32;

    use draw::style::style_for;
    use draw::text::Typeface;

    use super::{Pen, compose};
    use crate::layout::{Summary, layout};
    use crate::placement::Frame;
    use crate::testing::{DebugLabels, timeline};

    let typeface = Typeface::roboto().unwrap();
    let style = style_for(NonZeroU32::new(320).unwrap(), "9", &typeface);
    let timeline = timeline(40);
    let summary = Summary {
        name: "v",
        tracks,
        timeline: &timeline,
        ignored: &[],
        damaged: &[],
    };
    let frame = Frame {
        width: 320.0,
        height: 240.0,
    };
    let page = layout(&summary, frame, &style).remove(0);
    let pen = Pen {
        typeface: &typeface,
        style: &style,
        labels: &DebugLabels,
    };
    compose(&backdrop(Canvas::new(320, 240, GRAY).unwrap(), &[]), &page, pen).unwrap()
}

const GRAY: Rgb = Rgb::new(128, 128, 128);

fn walk(id: usize, start: [f64; 2], step: [f64; 2], count: usize) -> types::track::Track {
    let points: Vec<[f64; 2]> = (0..count)
        .map(|index| [start[0] + step[0] * index as f64, start[1] + step[1] * index as f64])
        .collect();
    crate::testing::track(id, 0, &points)
}

#[test]
fn composition_keeps_the_video_adds_the_legend_on_the_right_and_the_time_strip_below() {
    let image = gray_page(&[walk(1, [40.0, 200.0], [10.0, -5.0], 20)]);
    assert!(image.height() > 240);
    assert_eq!(at(&image, 5, 241), [32, 32, 32]);
    assert!(image.width() > 320);
    assert_eq!(at(&image, 5, 5), [77, 77, 77]);
    assert_eq!(at(&image, 320, 120), [80, 80, 80]);
}

#[test]
fn a_later_outline_does_not_cut_an_earlier_trajectory() {
    let crossing = [
        walk(1, [10.0, 120.0], [10.0, 0.0], 30),
        walk(2, [160.0, 5.0], [0.0, 10.0], 24),
    ];
    let image = gray_page(&crossing);
    let beside_the_crossing = at(&image, 162, 120);
    assert!(
        beside_the_crossing.iter().any(|&channel| channel > 100),
        "{beside_the_crossing:?}"
    );
}

fn page_with_one_marker(center: [f64; 2]) -> Canvas {
    use std::num::NonZeroU32;

    use draw::style::style_for;
    use draw::text::Typeface;

    use super::{Pen, compose};
    use crate::geometry::Trajectory;
    use crate::layout::{Mark, PageLayout};
    use crate::testing::{DebugLabels, track};

    let typeface = Typeface::roboto().unwrap();
    let style = style_for(NonZeroU32::new(320).unwrap(), "9", &typeface);
    let lone = track(1, 0, &[[100.0, 120.5]]);
    let page = PageLayout {
        minutes: None,
        header: Vec::new(),
        marks: vec![Mark {
            track: &lone,
            path: Trajectory::of(&lone),
            color: 3,
            start: "0:00".to_owned(),
            head: None,
            center,
            anchor: [100.0, 120.5],
            start_seconds: 0.0,
        }],
        window: [0.0, 4.0],
        ignored: Vec::new(),
        damaged: Vec::new(),
    };
    let pen = Pen {
        typeface: &typeface,
        style: &style,
        labels: &DebugLabels,
    };
    compose(&backdrop(Canvas::new(320, 240, GRAY).unwrap(), &[]), &page, pen).unwrap()
}

#[test]
fn a_marker_moved_off_its_track_is_tied_back_to_it_by_a_line_of_its_colour() {
    let green = draw::color::PALETTE[3];
    let tied = page_with_one_marker([200.0, 120.5]);
    assert_eq!(at(&tied, 150, 120), [green.red, green.green, green.blue]);
    let on_track = page_with_one_marker([100.0, 120.5]);
    assert_eq!(at(&on_track, 150, 120), [77, 77, 77]);
}

#[test]
fn the_tie_back_line_is_thinner_than_a_track_at_the_binoculars_width_so_it_does_not_read_as_a_flight() {
    use std::num::NonZeroU32;

    use draw::style::style_for;
    use draw::text::Typeface;

    let typeface = Typeface::roboto().unwrap();
    let style = style_for(NonZeroU32::new(1440).unwrap(), "99", &typeface);
    assert!(style.border < style.line);
    assert!(style.border + super::LEADER_HALO < style.outline);
}
