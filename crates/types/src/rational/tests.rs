use proptest::prelude::*;

use super::{Error, Rational, gcd};

#[test]
fn a_zero_denominator_is_refused() {
    assert_eq!(Rational::new(1, 0), Err(Error::ZeroDenominator));
}

#[test]
fn the_sign_lives_on_the_numerator() {
    let rate = Rational::new(3, -6).unwrap();
    assert_eq!((rate.num(), rate.den()), (-1, 2));
}

#[test]
fn a_rate_prints_as_ffmpeg_writes_it() {
    assert_eq!(Rational::new(20_000, 666).unwrap().to_string(), "10000/333");
}

#[test]
fn to_f64_matches_the_division() {
    let rate = Rational::new(90_110_000, 3_000_927).unwrap();
    assert!((rate.to_f64() - 30.027_388_203_711_72).abs() < 1e-12);
}

#[test]
fn terms_that_overflow_32_bits_once_reduced_are_refused() {
    assert_eq!(
        Rational::new(i32::MIN, -1),
        Err(Error::OutOfRange { num: i32::MIN, den: -1 })
    );
    assert_eq!(
        Rational::new(1, i32::MIN),
        Err(Error::OutOfRange { num: 1, den: i32::MIN })
    );
}

#[test]
fn the_most_negative_numerator_reduces_when_it_can() {
    let rate = Rational::new(i32::MIN, 2).unwrap();
    assert_eq!((rate.num(), rate.den()), (i32::MIN / 2, 1));
}

proptest! {
    #[test]
    fn equal_rates_compare_equal_whatever_their_terms(
        num in -10_000_i32..=10_000,
        den in -10_000_i32..=10_000,
        factor in -200_000_i32..=200_000,
    ) {
        prop_assume!(den != 0 && factor != 0);
        prop_assert_eq!(Rational::new(num, den), Rational::new(num * factor, den * factor));
    }

    #[test]
    fn any_pair_of_i32_gives_a_reduced_rate_or_an_error_never_a_panic(num: i32, den: i32) {
        match Rational::new(num, den) {
            Ok(rate) => {
                prop_assert!(rate.den() > 0);
                prop_assert_eq!(gcd(rate.num().unsigned_abs(), rate.den().unsigned_abs()), 1);
                prop_assert_eq!(i64::from(rate.num()) * i64::from(den), i64::from(num) * i64::from(rate.den()));
            }
            Err(Error::ZeroDenominator) => prop_assert_eq!(den, 0),
            Err(Error::OutOfRange { .. }) => prop_assert!(den == -1 || den == i32::MIN || num == i32::MIN),
        }
    }
}
