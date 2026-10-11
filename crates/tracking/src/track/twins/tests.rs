use proptest::prelude::*;
use types::detection::Detection;
use types::track::Track;

use super::{are_twins, combine, merge_twins};
use crate::fake::{detection, line};

fn track(points: Vec<Detection>) -> Track {
    Track::new(0, points).unwrap()
}

#[test]
fn fragments_of_one_animal_are_merged_with_one_point_per_frame() {
    let body = track(line(0, 10, (30.0, 150.0), (18.0, 0.0)));
    let wing = track(line(0, 10, (42.0, 168.0), (18.0, 0.0)));
    let merged = merge_twins(vec![body, wing], 36.0).unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].points().len(), 10);
}

#[test]
fn two_animals_flying_apart_are_not_merged() {
    let a = track(line(0, 10, (30.0, 150.0), (18.0, 0.0)));
    let b = track(line(0, 10, (42.0, 162.0), (18.0, 9.0)));
    assert_eq!(merge_twins(vec![a, b], 36.0).unwrap().len(), 2);
}

#[test]
fn tracks_that_never_coexist_are_not_twins() {
    let a = track(line(0, 5, (30.0, 150.0), (18.0, 0.0)));
    let b = track(line(10, 5, (36.0, 150.0), (18.0, 0.0)));
    assert!(!are_twins(&a, &b, 36.0));
}

#[test]
fn overlapping_tracks_without_a_shared_frame_are_not_twins() {
    let a = track(vec![detection(0, 0.0, 0.0), detection(2, 0.0, 0.0)]);
    let b = track(vec![detection(1, 0.0, 0.0), detection(3, 0.0, 0.0)]);
    assert!(!are_twins(&a, &b, 36.0));
}

#[test]
fn a_track_starting_on_the_last_frame_of_another_can_be_its_twin() {
    let earlier = track(line(0, 4, (0.0, 0.0), (10.0, 0.0)));
    let later = track(line(3, 4, (31.0, 0.0), (10.0, 0.0)));
    let merged = merge_twins(vec![earlier, later], 5.0).unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].points().len(), 7);
}

#[test]
fn twins_exactly_twin_distance_apart_are_merged() {
    let a = track(vec![detection(0, 0.0, 0.0), detection(1, 10.0, 0.0)]);
    let b = track(vec![detection(1, 13.0, 4.0), detection(2, 20.0, 0.0)]);
    assert!(are_twins(&a, &b, 5.0));
    assert!(!are_twins(&a, &b, 4.999));
}

#[test]
fn combined_fragment_is_area_weighted_and_boxes_both() {
    let big = Detection {
        area: 30.0,
        ..detection(0, 10.0, 10.0)
    };
    let small = Detection {
        area: 10.0,
        amplitude: 95.0,
        ..detection(0, 20.0, 30.0)
    };
    let merged = combine(big, small);
    assert!((merged.x - 12.5).abs() < 1e-12);
    assert!((merged.y - 15.0).abs() < 1e-12);
    assert_eq!((merged.top, merged.height), (9.0, 23.0));
    assert_eq!(
        (merged.area, merged.left, merged.width, merged.amplitude),
        (40.0, 9.0, 13.0, 95.0)
    );
}

#[test]
fn a_chain_of_twins_forms_one_group_in_place_of_its_first_member() {
    let first = track(line(0, 4, (0.0, 0.0), (10.0, 0.0)));
    let lone = track(line(1, 4, (0.0, 900.0), (10.0, 0.0)));
    let link = track(line(2, 4, (20.0, 20.0), (10.0, 0.0)));
    let close_to_link = track(line(2, 4, (25.0, 30.0), (10.0, 0.0)));
    let merged = merge_twins(vec![first, lone, close_to_link, link], 25.0).unwrap();
    let starts: Vec<(usize, usize)> = merged
        .iter()
        .map(|track| (track.first().frame.get(), track.points().len()))
        .collect();
    assert_eq!(starts, [(0, 6), (1, 4)]);
}

fn fragment() -> impl Strategy<Value = Detection> {
    (0.0..100.0_f64, 0.0..100.0_f64, 1.0..50.0_f64, 0.0..90.0_f64).prop_map(|(x, y, area, amplitude)| Detection {
        area,
        amplitude,
        ..detection(0, x, y)
    })
}

proptest! {
    #[test]
    fn a_combined_fragment_adds_areas_and_keeps_its_centre_in_the_box(a in fragment(), b in fragment()) {
        let merged = combine(a, b);
        prop_assert!((merged.area - a.area - b.area).abs() < 1e-9);
        prop_assert!(merged.left <= a.left.min(b.left) && merged.top <= a.top.min(b.top));
        prop_assert!(merged.left + merged.width >= (a.left + a.width).max(b.left + b.width));
        prop_assert!(merged.top + merged.height >= (a.top + a.height).max(b.top + b.height));
        prop_assert!(merged.x >= a.x.min(b.x) - 1e-9 && merged.x <= a.x.max(b.x) + 1e-9);
        prop_assert!(merged.y >= a.y.min(b.y) - 1e-9 && merged.y <= a.y.max(b.y) + 1e-9);
        prop_assert!((merged.amplitude - a.amplitude.max(b.amplitude)).abs() < 1e-12);
    }

    #[test]
    fn being_twins_does_not_depend_on_the_order(
        start in 0_usize..5, shift in 0.0..60.0_f64, length in 1_usize..8,
    ) {
        let a = track(line(0, 6, (0.0, 0.0), (10.0, 0.0)));
        let b = track(line(start, length, (shift, shift / 2.0), (10.0, 0.0)));
        prop_assert_eq!(are_twins(&a, &b, 36.0), are_twins(&b, &a, 36.0));
    }
}
