use draw::canvas::Canvas;
use draw::color::INK;
use draw::text::Typeface;
use media::convert::{Converter, SourceRange};
use media::frame::{ColorRange, YuvFrame};
use types::frame::FrameIndex;

use super::{Error, Picture};
use crate::config::RenderConfig;
use crate::overlay::Overlay;
use crate::testing::{thirty, track};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;

fn textured() -> YuvFrame {
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    for (index, value) in frame.y_mut().iter_mut().enumerate() {
        *value = (index % 251) as u8;
    }
    frame.u_mut().fill(120);
    frame.v_mut().fill(140);
    frame
}

fn overlay() -> Overlay {
    let tracks = [track(1, &[(0, 100.0, 150.0), (2, 120.0, 150.0)])];
    Overlay::new(
        &tracks,
        5,
        RenderConfig::default(),
        thirty(),
        &Typeface::roboto().unwrap(),
    )
}

#[test]
fn an_undrawn_frame_comes_out_through_the_display_tables_in_limited_range() {
    let source = textured();
    let mut picture = Picture::new(WIDTH, HEIGHT).unwrap();
    let drawn = picture.draw(&source, &overlay(), FrameIndex::new(4)).unwrap().clone();
    let converter = Converter::new(WIDTH, HEIGHT, SourceRange::FullBlackAt16).unwrap();
    let mut expected = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.display(&source, &mut expected).unwrap();
    assert_eq!(drawn, expected);
    assert_eq!(drawn.range(), ColorRange::Limited);
}

#[test]
fn the_drawn_pixels_come_out_as_the_round_trip_of_the_drawn_canvas() {
    let source = textured();
    let overlay = overlay();
    let mut picture = Picture::new(WIDTH, HEIGHT).unwrap();
    let undrawn = picture.draw(&source, &overlay, FrameIndex::new(4)).unwrap().clone();
    let drawn = picture.draw(&source, &overlay, FrameIndex::new(2)).unwrap().clone();
    let mut converter = Converter::new(WIDTH, HEIGHT, SourceRange::FullBlackAt16).unwrap();
    let mut canvas = Canvas::new(WIDTH, HEIGHT, INK).unwrap();
    converter.yuv_to_rgba(&source, canvas.rgba_mut()).unwrap();
    overlay.draw(&mut canvas, FrameIndex::new(2));
    let mut whole = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.rgba_to_yuv(canvas.rgba(), &mut whole).unwrap();
    let marked: Vec<usize> = (0..drawn.y().len())
        .filter(|index| drawn.y()[*index] != undrawn.y()[*index])
        .collect();
    assert!(marked.len() > 100);
    assert!(marked.iter().all(|index| drawn.y()[*index] == whole.y()[*index]));
}

#[test]
fn drawing_changes_only_the_pixels_inside_the_zones_of_the_marks() {
    let source = textured();
    let overlay = overlay();
    let mut picture = Picture::new(WIDTH, HEIGHT).unwrap();
    let undrawn = picture.draw(&source, &overlay, FrameIndex::new(4)).unwrap().clone();
    let drawn = picture.draw(&source, &overlay, FrameIndex::new(2)).unwrap().clone();
    let zones = overlay.zones(FrameIndex::new(2), WIDTH, HEIGHT);
    let changed: Vec<(u32, u32)> = drawn
        .y()
        .iter()
        .zip(undrawn.y())
        .enumerate()
        .filter(|(_, (after, before))| after != before)
        .map(|(index, _)| ((index % WIDTH as usize) as u32, (index / WIDTH as usize) as u32))
        .collect();
    assert!(changed.contains(&(103, 140)));
    let inside = |&(x, y): &(u32, u32)| {
        zones.iter().any(|zone| {
            (zone.left..zone.left + zone.width).contains(&x) && (zone.top..zone.top + zone.height).contains(&y)
        })
    };
    assert!(changed.iter().all(inside), "{zones:?}");
}

#[test]
fn a_frame_of_another_size_is_refused() {
    let mut picture = Picture::new(WIDTH, HEIGHT).unwrap();
    let small = YuvFrame::new(2, 2, ColorRange::Limited).unwrap();
    assert!(matches!(
        picture.draw(&small, &overlay(), FrameIndex::new(0)),
        Err(Error::Convert(_))
    ));
}

#[test]
fn an_empty_size_is_refused() {
    assert!(matches!(Picture::new(0, HEIGHT), Err(Error::Convert(_))));
}
