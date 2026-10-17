use draw::canvas::Canvas;
use draw::color::{INK, PALETTE, TRAIL};
use media::frame::{ColorRange, YuvFrame};
use types::detection::Detection;
use types::frame::FrameIndex;
use types::track::Track;

use super::{Crop, Magnifier, ZoomTrail, crop};
use crate::clips::Error;
use crate::picture::Marks;
use crate::testing::track;

const WIDTH: u32 = 1440;
const HEIGHT: u32 = 1080;

fn along_x(frames: std::ops::RangeInclusive<usize>, x0: f64, step: f64, y: f64) -> Track {
    let points: Vec<(usize, f64, f64)> = frames.map(|frame| (frame, x0 + step * frame as f64, y)).collect();
    track(1, &points)
}

fn patterned(width: u32, height: u32) -> YuvFrame {
    let mut frame = YuvFrame::new(width, height, ColorRange::Limited).unwrap();
    let fill = |plane: &mut [u8], row: usize, seed: usize| {
        for (index, value) in plane.iter_mut().enumerate() {
            *value = ((index % row) * 7 + (index / row) * 13 + seed) as u8;
        }
    };
    fill(frame.y_mut(), width as usize, 0);
    fill(frame.u_mut(), width.div_ceil(2) as usize, 50);
    fill(frame.v_mut(), width.div_ceil(2) as usize, 90);
    frame
}

fn chroma_of(luma: usize) -> usize {
    luma / 2
}

fn at(plane: &[u8], row: usize, x: usize, y: usize) -> u8 {
    plane[y * row + x]
}

#[test]
fn a_small_track_gets_the_smallest_frame_magnified_four_times_around_it() {
    let small = along_x(0..=2, 700.0, 10.0, 500.0);
    assert_eq!(
        crop(&small, WIDTH, HEIGHT),
        Crop {
            left: 530,
            top: 365,
            width: 360,
            height: 270
        }
    );
}

#[test]
fn the_frame_stays_inside_the_image_near_its_edges() {
    let left = crop(&along_x(0..=2, 10.0, 10.0, 20.0), WIDTH, HEIGHT);
    assert_eq!((left.left, left.top), (0, 0));
    let right = crop(&along_x(0..=2, 1420.0, 5.0, 1070.0), WIDTH, HEIGHT);
    assert_eq!((right.left + right.width, right.top + right.height), (WIDTH, HEIGHT));
}

#[test]
fn a_wide_track_gets_a_frame_in_whole_units_of_four_by_three_that_holds_it() {
    let wide = along_x(0..=10, 300.0, 50.0, 500.0);
    let frame = crop(&wide, WIDTH, HEIGHT);
    assert_eq!((frame.width, frame.height), (4 * 148, 3 * 148));
    assert!(frame.left <= 255 && frame.left + frame.width >= 845);
}

#[test]
fn a_spread_blob_does_not_widen_the_frame_once_its_box_is_bounded() {
    let mut points: Vec<Detection> = along_x(0..=2, 700.0, 10.0, 500.0).points().to_vec();
    let mut spread = points[1];
    (spread.left, spread.width) = (spread.x - 200.0, 400.0);
    points[1] = spread;
    let with_spread = Track::new(1, points).unwrap();
    assert_eq!(
        crop(&with_spread, WIDTH, HEIGHT),
        crop(&along_x(0..=2, 700.0, 10.0, 500.0), WIDTH, HEIGHT)
    );
}

#[test]
fn magnified_four_times_each_block_of_four_by_four_repeats_one_source_pixel() {
    let source = patterned(64, 48);
    let frame = Crop {
        left: 3,
        top: 5,
        width: 16,
        height: 12,
    };
    let mut into = None;
    let zoomed = Magnifier::new(frame, 64, 48).magnify(&source, &mut into).unwrap();
    for y in 0..48 {
        for x in 0..64 {
            assert_eq!(at(zoomed.y(), 64, x, y), at(source.y(), 64, 3 + x / 4, 5 + y / 4));
        }
    }
    for y in 0..24 {
        for x in 0..32 {
            let (source_x, source_y) = (chroma_of(3 + 2 * x / 4), chroma_of(5 + 2 * y / 4));
            assert_eq!(at(zoomed.u(), 32, x, y), at(source.u(), 32, source_x, source_y));
            assert_eq!(at(zoomed.v(), 32, x, y), at(source.v(), 32, source_x, source_y));
        }
    }
}

#[test]
fn magnified_three_times_from_an_odd_corner_each_pixel_takes_the_nearest_source_pixel() {
    let source = patterned(48, 36);
    let frame = Crop {
        left: 5,
        top: 3,
        width: 16,
        height: 12,
    };
    let mut into = None;
    let zoomed = Magnifier::new(frame, 48, 36).magnify(&source, &mut into).unwrap();
    for y in 0..36 {
        for x in 0..48 {
            assert_eq!(
                at(zoomed.y(), 48, x, y),
                at(source.y(), 48, 5 + x * 16 / 48, 3 + y * 12 / 36)
            );
        }
    }
    for y in 0..18 {
        for x in 0..24 {
            let (source_x, source_y) = (chroma_of(5 + 2 * x * 16 / 48), chroma_of(3 + 2 * y * 12 / 36));
            assert_eq!(at(zoomed.u(), 24, x, y), at(source.u(), 24, source_x, source_y));
        }
    }
}

#[test]
fn a_frame_covering_the_whole_image_gives_the_image_back() {
    let source = patterned(64, 48);
    let whole = Crop {
        left: 0,
        top: 0,
        width: 64,
        height: 48,
    };
    let mut into = None;
    let zoomed = Magnifier::new(whole, 64, 48).magnify(&source, &mut into).unwrap();
    assert_eq!(*zoomed, source);
}

fn trail() -> ZoomTrail {
    let frame = Crop {
        left: 0,
        top: 0,
        width: 360,
        height: 270,
    };
    ZoomTrail::new(&along_x(0..=10, 100.0, 10.0, 100.0), TRAIL, frame, (720, 540), 5)
}

#[test]
fn the_trail_is_cut_sixteen_pixels_around_the_target_and_placed_by_pixel_centers() {
    let pieces = trail().pieces(FrameIndex::new(10));
    let ends: Vec<(f64, f64)> = pieces.iter().map(|piece| (piece.from.x, piece.to.x)).collect();
    assert_eq!(ends, [(301.0, 321.0), (321.0, 341.0), (341.0, 359.0)]);
    let rows: Vec<(f64, f64)> = pieces.iter().map(|piece| (piece.from.y, piece.to.y)).collect();
    assert_eq!(rows, [(201.0, 201.0); 3]);
}

#[test]
fn a_frame_outside_the_track_has_no_trail() {
    assert_eq!(trail().pieces(FrameIndex::new(11)), []);
}

#[test]
fn a_still_point_draws_nothing_and_a_vertical_move_is_cut_like_any_other() {
    let points = [(0, 50.0, 40.0), (1, 50.0, 40.0), (2, 50.0, 60.0), (3, 50.0, 100.0)];
    let frame = Crop {
        left: 0,
        top: 0,
        width: 360,
        height: 270,
    };
    let zoom = ZoomTrail::new(&track(1, &points), TRAIL, frame, (360, 270), 5);
    let pieces = zoom.pieces(FrameIndex::new(3));
    let ends: Vec<(f64, f64)> = pieces.iter().map(|piece| (piece.from.y, piece.to.y)).collect();
    assert_eq!(ends, [(40.5, 60.5), (60.5, 79.5)]);
}

#[test]
fn the_zoom_trail_is_drawn_in_the_colour_of_its_track() {
    let frame = Crop {
        left: 0,
        top: 0,
        width: 360,
        height: 270,
    };
    let cyan = PALETTE[1];
    let zoom = ZoomTrail::new(&along_x(0..=10, 100.0, 10.0, 100.0), cyan, frame, (720, 540), 5);
    let mut canvas = Canvas::new(720, 540, INK).unwrap();
    zoom.draw(&mut canvas, FrameIndex::new(10));
    let drawn: Vec<&[u8; 4]> = canvas
        .rgba()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[..3] != [INK.red, INK.green, INK.blue])
        .collect();
    assert_ne!(drawn.len(), 0);
    assert!(drawn.iter().all(|pixel| pixel[2] > pixel[0]));
}

#[test]
fn every_drawn_trail_pixel_falls_in_a_zone() {
    let zoom = trail();
    for frame in 1..=10 {
        let frame = FrameIndex::new(frame);
        let mut canvas = Canvas::new(720, 540, INK).unwrap();
        zoom.draw(&mut canvas, frame);
        let zones = zoom.zones(frame, 720, 540);
        for (index, pixel) in canvas.rgba().as_chunks::<4>().0.iter().enumerate() {
            if pixel[..3] == [INK.red, INK.green, INK.blue] {
                continue;
            }
            let (x, y) = ((index % 720) as u32, (index / 720) as u32);
            let inside = zones.iter().any(|zone| {
                (zone.left..zone.left + zone.width).contains(&x) && (zone.top..zone.top + zone.height).contains(&y)
            });
            assert!(inside, "pixel {x},{y} of frame {frame:?} is outside every zone");
        }
    }
}

#[test]
fn a_segment_that_crosses_the_target_is_cut_in_two_around_it() {
    let points = [(0, 20.0, 50.0), (1, 100.0, 50.0), (2, 60.0, 50.0)];
    let frame = Crop {
        left: 0,
        top: 0,
        width: 360,
        height: 270,
    };
    let zoom = ZoomTrail::new(&track(1, &points), TRAIL, frame, (360, 270), 5);
    let ends: Vec<(f64, f64)> = zoom
        .pieces(FrameIndex::new(2))
        .iter()
        .map(|piece| (piece.from.x, piece.to.x))
        .collect();
    assert_eq!(ends, [(20.5, 39.5), (81.5, 100.5), (100.5, 81.5)]);
}

#[test]
fn a_frame_of_another_size_than_the_zoom_was_made_for_is_refused() {
    let whole = Crop {
        left: 0,
        top: 0,
        width: 64,
        height: 48,
    };
    let mut into = None;
    let refused = Magnifier::new(whole, 64, 48).magnify(&patterned(32, 24), &mut into);
    assert!(matches!(
        refused,
        Err(Error::ZoomSize {
            width: 64,
            height: 48,
            found_width: 32,
            found_height: 24
        })
    ));
}
