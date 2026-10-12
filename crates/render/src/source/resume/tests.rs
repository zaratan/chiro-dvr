use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;

use super::clean_key_before;
use crate::source::numbering::Numbering;

fn frames(indices: &[usize]) -> Vec<FrameIndex> {
    indices.iter().copied().map(FrameIndex::new).collect()
}

fn every_frame_timed(count: i64) -> Timeline {
    let rate = FrameRate::new(Rational::new(30, 1).unwrap()).unwrap();
    let pts = (0..count).map(|frame| Some(Pts::new(frame * 100))).collect();
    Timeline::new(pts, Rational::new(1, 3000).unwrap(), rate).unwrap()
}

fn resume(first: usize, keys: &[usize], errors: &[usize], timeline: &Timeline) -> Option<usize> {
    let numbering = Numbering::of(timeline).unwrap();
    clean_key_before(FrameIndex::new(first), &frames(keys), &frames(errors), &numbering).map(|(key, pts)| {
        assert_eq!(timeline.pts(key), Some(pts));
        key.get()
    })
}

#[test]
fn the_reading_resumes_at_the_last_key_frame_at_or_before_the_first_wanted_frame() {
    let timeline = every_frame_timed(32);
    let keys = [0, 8, 16, 24];
    assert_eq!(resume(20, &keys, &[], &timeline), Some(16));
    assert_eq!(resume(16, &keys, &[], &timeline), Some(16));
    assert_eq!(resume(7, &keys, &[], &timeline), Some(0));
    assert_eq!(resume(31, &keys, &[], &timeline), Some(24));
}

#[test]
fn a_key_frame_in_error_whose_concealment_draws_on_earlier_frames_sends_the_reading_back_to_the_key_frame_before_it() {
    let timeline = every_frame_timed(32);
    assert_eq!(resume(20, &[0, 8, 16, 24], &[16, 17], &timeline), Some(8));
    assert_eq!(resume(20, &[0, 8, 16, 24], &[8, 16], &timeline), Some(0));
}

#[test]
fn without_a_clean_key_frame_before_the_window_the_reading_starts_from_the_beginning() {
    let timeline = every_frame_timed(32);
    assert_eq!(resume(3, &[5, 10], &[], &timeline), None);
    assert_eq!(resume(12, &[0, 8], &[0, 8], &timeline), None);
    assert_eq!(resume(12, &[], &[], &timeline), None);
}

#[test]
fn a_key_frame_without_pts_cannot_be_sought_and_is_passed_over() {
    let rate = FrameRate::new(Rational::new(30, 1).unwrap()).unwrap();
    let pts = vec![Some(Pts::new(0)), None, Some(Pts::new(200))];
    let timeline = Timeline::new(pts, Rational::new(1, 3000).unwrap(), rate).unwrap();
    assert_eq!(resume(2, &[0, 1], &[], &timeline), Some(0));
}
