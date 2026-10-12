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
fn an_undrawn_frame_comes_out_as_a_full_range_round_trip_in_limited_range() {
    let source = textured();
    let mut picture = Picture::new(WIDTH, HEIGHT).unwrap();
    let drawn = picture.draw(&source, &overlay(), FrameIndex::new(4)).unwrap().clone();
    let mut converter = Converter::new(WIDTH, HEIGHT, SourceRange::Full).unwrap();
    let mut rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
    let mut expected = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.yuv_to_rgba(&source, &mut rgba).unwrap();
    converter.rgba_to_yuv(&rgba, &mut expected).unwrap();
    assert_eq!(drawn, expected);
    assert_eq!(drawn.range(), ColorRange::Limited);
}

#[test]
fn drawing_changes_only_the_pixels_around_the_marks() {
    let source = textured();
    let mut picture = Picture::new(WIDTH, HEIGHT).unwrap();
    let undrawn = picture.draw(&source, &overlay(), FrameIndex::new(4)).unwrap().clone();
    let drawn = picture.draw(&source, &overlay(), FrameIndex::new(2)).unwrap().clone();
    let changed: Vec<(usize, usize)> = drawn
        .y()
        .iter()
        .zip(undrawn.y())
        .enumerate()
        .filter(|(_, (after, before))| after != before)
        .map(|(index, _)| (index % WIDTH as usize, index / WIDTH as usize))
        .collect();
    assert!(changed.contains(&(103, 140)));
    assert!(
        changed
            .iter()
            .all(|&(x, y)| (95..=150).contains(&x) && (100..=172).contains(&y)),
        "{changed:?}"
    );
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
