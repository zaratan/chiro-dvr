use super::{assign_colors, path_distance};
use crate::testing::horizontal;

#[test]
fn path_distance_is_the_closest_approach() {
    assert_eq!(path_distance(horizontal(0.0).points(), horizontal(30.0).points()), 30.0);
}

#[test]
fn neighbours_never_share_a_color_even_past_the_palette_size() {
    let stacked: Vec<_> = (0..4).map(|row| horizontal(10.0 * f64::from(row))).collect();
    let colors = assign_colors(&stacked, 3, 15.0);
    assert!(colors.windows(2).all(|pair| pair[0] != pair[1]), "{colors:?}");
}

#[test]
fn distant_tracks_spread_over_the_palette_before_reusing_colors() {
    let far_apart: Vec<_> = (0..4).map(|row| horizontal(1000.0 * f64::from(row))).collect();
    assert_eq!(assign_colors(&far_apart, 4, 15.0), [0, 1, 2, 3]);
}

#[test]
fn close_tracks_take_the_color_least_used_around_them() {
    let paths = [horizontal(0.0), horizontal(1000.0), horizontal(5.0)];
    assert_eq!(assign_colors(&paths, 2, 15.0), [0, 1, 1]);
}

#[test]
fn an_empty_palette_gives_colour_zero_rather_than_failing() {
    assert_eq!(assign_colors(&[horizontal(0.0), horizontal(5.0)], 0, 15.0), [0, 0]);
}
