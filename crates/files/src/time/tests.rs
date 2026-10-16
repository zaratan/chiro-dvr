use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::span::Span;
use types::timeline::Timeline;

use super::{clock, frame_duration, frame_seconds, hundredths, span_seconds};

fn timeline(pts: Vec<Option<Pts>>) -> Timeline {
    let rate = FrameRate::new(Rational::new(10_000, 333).unwrap()).unwrap();
    Timeline::new(pts, Rational::new(1, 10_000).unwrap(), rate).unwrap()
}

fn regular(frames: i64) -> Timeline {
    timeline((0..frames).map(|frame| Some(Pts::new(frame * 333))).collect())
}

fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

#[test]
fn a_clock_shows_hours_minutes_and_seconds_so_a_spreadsheet_never_reads_minutes_as_hours() {
    assert_eq!(clock(7.39), "0:00:07");
    assert_eq!(clock(237.18), "0:03:57");
    assert_eq!(clock(3725.0), "1:02:05");
}

#[test]
fn a_clock_drops_the_hundredths_like_the_name_of_the_clip_instead_of_rounding_up() {
    assert_eq!(clock(4.8), "0:00:04");
    assert_eq!(clock(59.996), "0:00:59");
}

#[test]
fn a_negative_time_shows_as_zero() {
    assert_eq!(clock(-0.2), "0:00:00");
}

#[test]
fn seconds_round_to_the_hundredth() {
    assert!((hundredths(23.676) - 23.68).abs() < 1e-12);
}

#[test]
fn a_frame_takes_the_time_of_its_pts() {
    assert!((frame_seconds(&regular(2000), FrameIndex::new(1019)) - 33.9327).abs() < 1e-9);
}

#[test]
fn a_frame_without_pts_after_a_gap_keeps_the_clock_of_the_pts_before_it() {
    let line = timeline(vec![Some(Pts::new(0)), Some(Pts::new(14 * 333)), None, None]);
    assert!((frame_seconds(&line, FrameIndex::new(3)) - 16.0 * 0.0333).abs() < 1e-9);
}

#[test]
fn a_video_without_any_pts_falls_back_on_the_rate() {
    let line = timeline(vec![None, None]);
    assert!((frame_seconds(&line, FrameIndex::new(1)) - 0.0333).abs() < 1e-9);
    assert!((frame_duration(&line) - 0.0333).abs() < 1e-9);
    assert!((frame_seconds(&timeline(vec![]), FrameIndex::new(2)) - 0.0666).abs() < 1e-9);
}

#[test]
fn a_span_ends_where_the_frame_after_it_starts() {
    let [start, end] = span_seconds(span(10, 19), &regular(100));
    assert!((start - 0.333).abs() < 1e-9 && (end - 0.666).abs() < 1e-9);
}

#[test]
fn a_span_reaching_the_last_frame_ends_one_frame_later() {
    let [_, end] = span_seconds(span(95, 99), &regular(100));
    assert!((end - 3.33).abs() < 1e-9);
}

#[test]
fn a_span_over_frames_without_pts_stays_in_order() {
    let line = timeline(vec![
        Some(Pts::new(0)),
        Some(Pts::new(14 * 333)),
        None,
        None,
        Some(Pts::new(17 * 333)),
    ]);
    let [start, end] = span_seconds(span(2, 3), &line);
    assert!(start < end);
    assert!((end - 17.0 * 0.0333).abs() < 1e-9);
}
