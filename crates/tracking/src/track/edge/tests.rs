use super::at_edge;
use crate::fake::{detection, size};

#[test]
fn a_point_closer_than_thirty_pixels_to_any_side_is_at_the_edge() {
    for (x, y) in [(29.9, 500.0), (1410.1, 500.0), (700.0, 29.9), (700.0, 1050.1)] {
        assert!(at_edge(&detection(0, x, y), size()), "{x}, {y}");
    }
}

#[test]
fn a_point_thirty_pixels_inside_every_side_is_not_at_the_edge() {
    for (x, y) in [(30.0, 500.0), (1410.0, 500.0), (700.0, 30.0), (700.0, 1050.0)] {
        assert!(!at_edge(&detection(0, x, y), size()), "{x}, {y}");
    }
}
