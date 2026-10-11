use super::{narrowed, numbered, real, wide};

#[test]
fn every_value_the_detection_converts_stays_exact() {
    assert_eq!(numbered(960 * 720 - 1), 691_199);
    assert!((real(691_200 * 960) - 663_552_000.0).abs() < f64::EPSILON);
    assert!((wide(-1_100_000_000_000_000) + 1.1e15).abs() < f64::EPSILON);
    assert!((narrowed(8.0 * 1.4826) - 11.8608).abs() < 1e-5);
}
