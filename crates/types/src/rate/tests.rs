use std::num::NonZeroU32;

use super::{Error, FrameRate};
use crate::frame::FrameIndex;
use crate::rational::{DivisionError, Rational};

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

#[test]
fn the_slowed_rate_of_the_092_is_exactly_a_quarter() {
    let rate = FrameRate::new(Rational::new(90_110_000, 3_000_927).unwrap()).unwrap();
    let four = NonZeroU32::new(4).unwrap();
    assert_eq!(
        rate.slowed(four).unwrap().get(),
        Rational::new(22_527_500, 3_000_927).unwrap()
    );
}

#[test]
fn a_denominator_near_the_limit_slows_when_the_numerator_takes_the_factor() {
    let rate = FrameRate::new(Rational::new(8, i32::MAX).unwrap()).unwrap();
    let four = NonZeroU32::new(4).unwrap();
    assert_eq!(rate.slowed(four).unwrap().get(), Rational::new(2, i32::MAX).unwrap());
}

#[test]
fn a_slowed_rate_that_overflows_is_refused() {
    let rate = FrameRate::new(Rational::new(3, 1_000_000_000).unwrap()).unwrap();
    let four = NonZeroU32::new(4).unwrap();
    assert_eq!(
        rate.slowed(four),
        Err(DivisionError {
            value: Rational::new(3, 1_000_000_000).unwrap(),
            factor: 4
        })
    );
}
