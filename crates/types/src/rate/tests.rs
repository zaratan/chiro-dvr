use super::{Error, FrameRate};
use crate::frame::FrameIndex;
use crate::rational::Rational;

#[test]
fn a_rate_that_is_not_positive_is_refused() {
    let negative = Rational::new(-30, 1).unwrap();
    assert_eq!(FrameRate::new(negative), Err(Error(negative)));
    assert_eq!(Error(negative).to_string(), "the frame rate -30/1 is not positive");
    assert!(FrameRate::new(Rational::new(0, 1).unwrap()).is_err());
}

#[test]
fn the_first_track_of_the_092_starts_at_7_39_seconds() {
    let rate = FrameRate::new(Rational::new(90_110_000, 3_000_927).unwrap()).unwrap();
    assert_eq!(rate.get(), Rational::new(90_110_000, 3_000_927).unwrap());
    assert!((rate.seconds(FrameIndex::new(222)) - 7.393_250_405).abs() < 1e-9);
}
