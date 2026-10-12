use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;

use super::Numbering;
use crate::source::Error;

fn timeline(pts: &[Option<i64>]) -> Timeline {
    let rate = FrameRate::new(Rational::new(30, 1).unwrap()).unwrap();
    Timeline::new(
        pts.iter().map(|pts| pts.map(Pts::new)).collect(),
        Rational::new(1, 3000).unwrap(),
        rate,
    )
    .unwrap()
}

#[test]
fn a_frame_is_found_by_its_pts_and_frames_without_pts_keep_their_number() {
    let numbering = Numbering::of(&timeline(&[Some(0), None, Some(200), Some(300)])).unwrap();
    assert_eq!(numbering.frame(Pts::new(0)), Some(FrameIndex::new(0)));
    assert_eq!(numbering.frame(Pts::new(200)), Some(FrameIndex::new(2)));
    assert_eq!(numbering.frame(Pts::new(300)), Some(FrameIndex::new(3)));
    assert_eq!(numbering.pts(FrameIndex::new(2)), Some(Pts::new(200)));
    assert_eq!(numbering.pts(FrameIndex::new(1)), None);
    assert_eq!(numbering.pts(FrameIndex::new(4)), None);
}

#[test]
fn a_pts_the_timeline_does_not_hold_finds_no_frame() {
    let numbering = Numbering::of(&timeline(&[Some(0), Some(200)])).unwrap();
    assert_eq!(numbering.frame(Pts::new(100)), None);
    assert_eq!(numbering.frame(Pts::new(-100)), None);
}

#[test]
fn a_repeated_or_falling_pts_is_refused_with_the_frame_that_breaks_the_order() {
    assert!(matches!(
        Numbering::of(&timeline(&[Some(0), Some(200), None, Some(200)])),
        Err(Error::UnorderedPts { frame }) if frame == FrameIndex::new(3)
    ));
    assert!(matches!(
        Numbering::of(&timeline(&[Some(100), Some(0)])),
        Err(Error::UnorderedPts { frame }) if frame == FrameIndex::new(1)
    ));
}
