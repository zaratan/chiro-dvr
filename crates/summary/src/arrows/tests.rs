use super::arrow_head;
use crate::geometry::distance;
use crate::testing::path;

#[test]
fn arrow_tip_is_the_last_point_and_wings_point_backwards() {
    let [tip, left, right] = arrow_head(&path(&[[0.0, 0.0], [100.0, 0.0]]), 20.0).unwrap();
    assert_eq!(tip, [100.0, 0.0]);
    assert!(left[0] < 100.0 && right[0] < 100.0);
    assert!(left[1] * right[1] < 0.0);
}

#[test]
fn arrow_keeps_its_size_when_the_last_step_is_tiny() {
    let [tip, wing, _] = arrow_head(&path(&[[0.0, 0.0], [100.0, 0.0], [100.5, 0.0]]), 20.0).unwrap();
    assert!((distance(tip, wing) - 20.0).abs() < 1e-9);
}

#[test]
fn track_that_never_moves_has_no_arrow() {
    assert_eq!(arrow_head(&path(&[[5.0, 5.0], [5.0, 5.0]]), 20.0), None);
}
