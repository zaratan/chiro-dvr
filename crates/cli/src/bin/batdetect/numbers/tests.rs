use super::{english, french};

#[test]
fn french_writes_a_decimal_comma() {
    assert_eq!(french("1.5"), "1,5");
    assert_eq!(french("0.25"), "0,25");
}

#[test]
fn french_groups_the_thousands_with_a_narrow_no_break_space() {
    assert_eq!(french("999"), "999");
    assert_eq!(french("1234.5"), "1\u{202f}234,5");
    assert_eq!(french("1234567"), "1\u{202f}234\u{202f}567");
    assert_eq!(french("-1234"), "-1\u{202f}234");
}

#[test]
fn english_keeps_the_point_and_no_separator() {
    assert_eq!(english("1.5"), "1.5");
    assert_eq!(english("1234.5"), "1234.5");
    assert_eq!(english("-1234567"), "-1234567");
}

#[test]
fn minus_zero_is_written_zero() {
    assert_eq!(french("-0"), "0");
    assert_eq!(english("-0"), "0");
}

#[test]
fn what_is_not_a_plain_number_is_left_as_given() {
    for text in ["NaN", "inf", "-inf", "", "-", "1e5"] {
        assert_eq!(french(text), text);
        assert_eq!(english(text), text);
    }
}
