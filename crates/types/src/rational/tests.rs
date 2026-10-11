use proptest::prelude::*;

use super::{Error, ParseError, Rational, gcd};

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
    fn a_printed_rate_reads_back_to_itself(num: i32, den: i32) {
        if let Ok(rate) = Rational::new(num, den) {
            prop_assert_eq!(rate.to_string().parse::<Rational>(), Ok(rate));
        }
    }

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

#[test]
fn a_rate_reads_as_ffprobe_writes_it() {
    assert_eq!(
        "90110000/3000927".parse(),
        Rational::new(90_110_000, 3_000_927).map_err(ParseError::from)
    );
}

#[test]
fn a_rate_without_a_slash_is_refused() {
    let error = "30".parse::<Rational>().unwrap_err();
    assert_eq!(error, ParseError::NoSlash("30".to_owned()));
    assert_eq!(error.to_string(), "\"30\" is not written as num/den");
}

#[test]
fn a_term_that_is_not_an_integer_is_refused() {
    assert!(matches!("x/1".parse::<Rational>(), Err(ParseError::Term { .. })));
    assert!(matches!(
        "1/9999999999".parse::<Rational>(),
        Err(ParseError::Term { .. })
    ));
}

#[test]
fn an_unknown_rate_written_0_over_0_is_refused() {
    assert_eq!(
        "0/0".parse::<Rational>(),
        Err(ParseError::Value(Error::ZeroDenominator))
    );
}

#[test]
fn a_rate_is_a_json_string() {
    let rate: Rational = serde_json::from_str("\"10000/333\"").unwrap();
    assert_eq!(serde_json::to_string(&rate).unwrap(), "\"10000/333\"");
    assert!(serde_json::from_str::<Rational>("\"30\"").is_err());
}
