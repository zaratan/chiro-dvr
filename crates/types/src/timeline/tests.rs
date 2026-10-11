use super::{Error, Timeline};
use crate::frame::{FrameIndex, Pts};
use crate::rate::FrameRate;
use crate::rational::Rational;

fn rate_092() -> FrameRate {
    FrameRate::new(Rational::new(90_110_000, 3_000_927).unwrap()).unwrap()
}

fn timeline(pts: Vec<Option<Pts>>) -> Timeline {
    Timeline::new(pts, Rational::new(1, 10_000).unwrap(), rate_092()).unwrap()
}

#[test]
fn a_time_base_that_is_not_positive_is_refused() {
    let zero = Rational::new(0, 1).unwrap();
    assert_eq!(Timeline::new(vec![], zero, rate_092()), Err(Error(zero)));
    assert_eq!(Error(zero).to_string(), "the time base 0/1 is not positive");
}

#[test]
fn the_pts_of_a_frame_converts_to_seconds_through_the_time_base() {
    let line = timeline(vec![Some(Pts::new(0)), Some(Pts::new(333)), Some(Pts::new(2_094_237))]);
    assert!((line.seconds_by_pts(FrameIndex::new(1)).unwrap() - 0.0333).abs() < 1e-12);
    assert!((line.seconds_by_pts(FrameIndex::new(2)).unwrap() - 209.4237).abs() < 1e-9);
}

#[test]
fn a_frame_without_pts_or_past_the_end_has_no_pts_time() {
    let line = timeline(vec![None]);
    assert_eq!(line.pts(FrameIndex::new(0)), None);
    assert_eq!(line.seconds_by_pts(FrameIndex::new(0)), None);
    assert_eq!(line.seconds_by_pts(FrameIndex::new(1)), None);
}

#[test]
fn the_rate_time_does_not_need_a_pts() {
    let line = timeline(vec![]);
    assert!((line.seconds_by_rate(FrameIndex::new(222)) - 7.393_250_405).abs() < 1e-9);
}

#[test]
fn a_timeline_keeps_one_entry_per_frame_with_its_time_base_and_rate() {
    let line = timeline(vec![Some(Pts::new(0)), None]);
    assert_eq!(line.frame_count(), 2);
    assert_eq!(line.time_base(), Rational::new(1, 10_000).unwrap());
    assert_eq!(line.rate(), rate_092());
}
