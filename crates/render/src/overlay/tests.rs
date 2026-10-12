use draw::canvas::Canvas;
use draw::color::{BOX, INK, Rgb, TRAIL};
use draw::text::Typeface;
use types::detection::Detection;
use types::frame::FrameIndex;
use types::track::Track;

use super::Overlay;
use crate::config::RenderConfig;
use crate::testing::{thirty, track};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;

fn drawn(tracks: &[Track], config: RenderConfig, frame: usize) -> Canvas {
    let typeface = Typeface::roboto().unwrap();
    let overlay = Overlay::new(tracks, 10, config, thirty(), &typeface);
    let mut canvas = Canvas::new(WIDTH, HEIGHT, INK).unwrap();
    overlay.draw(&mut canvas, FrameIndex::new(frame));
    canvas
}

fn pixel(canvas: &Canvas, x: u32, y: u32) -> Rgb {
    let at = ((y * WIDTH + x) * 4) as usize;
    let rgba = &canvas.rgba()[at..at + 3];
    Rgb::new(rgba[0], rgba[1], rgba[2])
}

fn reddish(canvas: &Canvas, columns: std::ops::RangeInclusive<u32>, rows: std::ops::RangeInclusive<u32>) -> bool {
    rows.flat_map(|y| columns.clone().map(move |x| (x, y)))
        .any(|(x, y)| pixel(canvas, x, y).red > 0)
}

#[test]
fn a_detected_point_gets_a_two_pixel_box_and_an_interpolated_one_a_one_pixel_box() {
    let tracks = [track(1, &[(0, 100.0, 150.0), (4, 140.0, 150.0)])];
    let detected = drawn(&tracks, RenderConfig::default(), 0);
    let row: Vec<Rgb> = (81..=84).map(|x| pixel(&detected, x, 140)).collect();
    assert_eq!(row, [INK, BOX, BOX, INK]);
    let interpolated = drawn(&tracks, RenderConfig::default(), 2);
    let row: Vec<Rgb> = (102..=104).map(|x| pixel(&interpolated, x, 140)).collect();
    assert_eq!(row, [INK, BOX, INK]);
}

#[test]
fn the_box_reaches_the_pad_beyond_the_detection_on_every_side() {
    let tracks = [track(1, &[(0, 100.0, 150.0)])];
    let canvas = drawn(&tracks, RenderConfig::new(20, 1.0, 1.5, 300).unwrap(), 0);
    assert_eq!(pixel(&canvas, 75, 150), BOX);
    assert_eq!(pixel(&canvas, 125, 150), BOX);
    assert_eq!(pixel(&canvas, 100, 125), BOX);
    assert_eq!(pixel(&canvas, 100, 175), BOX);
    assert_eq!(pixel(&canvas, 76, 150), INK);
}

#[test]
fn every_track_present_on_the_frame_is_drawn() {
    let tracks = [track(1, &[(0, 100.0, 150.0)]), track(2, &[(0, 300.0, 150.0)])];
    let canvas = drawn(&tracks, RenderConfig::default(), 0);
    assert_eq!(pixel(&canvas, 83, 140), BOX);
    assert_eq!(pixel(&canvas, 283, 140), BOX);
}

#[test]
fn the_trail_joins_the_detected_points_of_the_last_trail_frames_to_the_current_one() {
    let points: Vec<(usize, f64, f64)> = (0..6)
        .map(|frame| (frame, 100.0 + 10.0 * frame as f64, 150.0))
        .collect();
    let two_frames = RenderConfig::new(12, 2.0 / 30.0, 1.5, 300).unwrap();
    let canvas = drawn(&[track(1, &points)], two_frames, 5);
    assert_eq!(pixel(&canvas, 135, 150), TRAIL);
    assert_eq!(pixel(&canvas, 125, 150), INK);
}

#[test]
fn a_track_seen_once_draws_no_trail() {
    let canvas = drawn(&[track(1, &[(0, 100.0, 150.0)])], RenderConfig::default(), 0);
    assert!(canvas.rgba().as_chunks::<4>().0.iter().all(|pixel| pixel[1] == 0));
}

#[test]
fn the_track_number_stands_six_pixels_above_its_box() {
    let canvas = drawn(&[track(7, &[(0, 100.0, 150.0)])], RenderConfig::default(), 0);
    assert!(reddish(&canvas, 83..=120, 105..=127));
    assert!(!reddish(&canvas, 83..=120, 128..=131));
}

#[test]
fn a_frame_without_a_track_or_beyond_the_video_is_left_as_it_was() {
    let blank = Canvas::new(WIDTH, HEIGHT, INK).unwrap();
    let tracks = [track(1, &[(0, 100.0, 150.0)])];
    assert_eq!(drawn(&tracks, RenderConfig::default(), 3), blank);
    assert_eq!(drawn(&tracks, RenderConfig::default(), 50), blank);
}

fn spread(track: &Track, frame: usize, left: f64, width: f64, top: f64, height: f64) -> Track {
    let points = track
        .points()
        .iter()
        .map(|point| {
            if point.frame.get() == frame {
                Detection {
                    left,
                    width,
                    top,
                    height,
                    ..*point
                }
            } else {
                *point
            }
        })
        .collect();
    Track::new(track.id(), points).unwrap()
}

#[test]
fn a_box_spread_far_beyond_the_usual_size_of_its_track_is_drawn_four_medians_wide_around_the_detection() {
    let steady = track(1, &[(0, 200.0, 150.0), (1, 200.0, 150.0), (2, 200.0, 150.0)]);
    let tracks = [spread(&steady, 1, 15.0, 190.0, 140.0, 20.0)];
    let canvas = drawn(&tracks, RenderConfig::default(), 1);
    assert_eq!(pixel(&canvas, 168, 150), BOX);
    assert_eq!(pixel(&canvas, 232, 150), BOX);
    assert_eq!(pixel(&canvas, 3, 150), INK);
    assert_eq!(pixel(&canvas, 200, 128), BOX);
}

#[test]
fn a_box_within_four_medians_of_its_track_keeps_its_place_and_size() {
    let steady = track(1, &[(0, 200.0, 150.0), (1, 200.0, 150.0), (2, 200.0, 150.0)]);
    let tracks = [spread(&steady, 1, 170.0, 40.0, 145.0, 10.0)];
    let canvas = drawn(&tracks, RenderConfig::default(), 1);
    assert_eq!(pixel(&canvas, 158, 150), BOX);
    assert_eq!(pixel(&canvas, 222, 150), BOX);
}

#[test]
fn the_median_of_an_even_number_of_boxes_is_the_mean_of_the_two_middle_ones() {
    let steady = track(
        1,
        &[
            (0, 200.0, 150.0),
            (1, 200.0, 150.0),
            (2, 200.0, 150.0),
            (3, 200.0, 150.0),
        ],
    );
    let middle = spread(&steady, 2, 185.0, 30.0, 145.0, 10.0);
    let tracks = [spread(&middle, 3, 15.0, 190.0, 145.0, 10.0)];
    let canvas = drawn(&tracks, RenderConfig::default(), 3);
    assert_eq!(pixel(&canvas, 148, 150), BOX);
    assert_eq!(pixel(&canvas, 252, 150), BOX);
}
