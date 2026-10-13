use engine::gray::gray;
use engine::luma::LumaMut;
use types::frame::FrameIndex;

use super::{Injector, Window, gray_change, inject, nearest_luma, nearest_lumas, window};
use crate::synthetic::{Class, Motion, Point, Target};

const WIDTH: usize = 40;
const HEIGHT: usize = 30;
const BACKGROUND: u8 = 126;

fn target(id: usize, positions: Vec<Point>, amplitude: f64, sigma: f64) -> Target {
    let amplitudes = vec![amplitude; positions.len()];
    Target::new(
        id,
        FrameIndex::new(0),
        positions,
        amplitudes,
        Class::new(amplitude, sigma, Motion::Pass).unwrap(),
    )
    .unwrap()
}

fn plane(level: u8) -> Vec<u8> {
    vec![level; WIDTH * HEIGHT]
}

fn injected(pixels: &mut [u8], target: &Target) -> Option<f64> {
    inject(&mut LumaMut::new(pixels, WIDTH).unwrap(), target, FrameIndex::new(0))
}

fn realized(before: u8, after: u8) -> i32 {
    i32::from(gray(after)) - i32::from(gray(before))
}

#[test]
fn the_nearest_luma_of_a_gray_the_table_reaches_gives_back_that_gray() {
    for luma in 0..=u8::MAX {
        assert_eq!(gray(nearest_luma(gray(luma))), gray(luma), "{luma}");
    }
}

#[test]
fn every_gray_is_reached_within_one_level() {
    for aimed in 0..=u8::MAX {
        assert!(gray(nearest_luma(aimed)).abs_diff(aimed) <= 1, "{aimed}");
    }
}

#[test]
fn a_gray_the_table_skips_goes_alternately_one_level_darker_and_one_level_brighter() {
    let table = nearest_lumas();
    let reached = |aimed: u8| gray(table[usize::from(aimed)]);
    let skipped: Vec<i16> = (0..=u8::MAX)
        .filter(|&aimed| reached(aimed) != aimed)
        .map(|aimed| i16::from(reached(aimed)) - i16::from(aimed))
        .collect();

    assert!(skipped.len() > 30, "{skipped:?}");
    assert!(skipped.iter().step_by(2).all(|&gap| gap == -1), "{skipped:?}");
    assert!(skipped.iter().skip(1).step_by(2).all(|&gap| gap == 1), "{skipped:?}");
}

#[test]
fn the_injected_gray_is_within_one_level_of_the_floor_of_the_gaussian_and_nothing_moves_outside_its_window() {
    for sigma in [1.5, 3.0, 5.0] {
        for center in [Point { x: 20.0, y: 15.0 }, Point { x: 20.5, y: 14.5 }] {
            let mut pixels = plane(BACKGROUND);
            let area = window(center, sigma, WIDTH, HEIGHT).unwrap();

            injected(&mut pixels, &target(0, vec![center], -40.0, sigma)).unwrap();

            for (index, &after) in pixels.iter().enumerate() {
                let (x, y) = (index % WIDTH, index / WIDTH);
                let inside = (area.left..area.right).contains(&x) && (area.top..area.bottom).contains(&y);
                let expected = if inside {
                    gray_change(-40.0, sigma, center, x, y)
                } else {
                    0.0
                };
                let change = f64::from(realized(BACKGROUND, after));
                assert!(
                    (change - expected).abs() <= 1.0,
                    "σ {sigma} {center:?} ({x}, {y}): {change} for {expected}"
                );
                assert!(inside || change == 0.0);
            }
        }
    }
}

#[test]
fn the_window_reaches_three_sigma_rounded_up_around_the_floored_center() {
    assert_eq!(
        window(Point { x: 20.7, y: 15.2 }, 1.5, WIDTH, HEIGHT),
        Some(Window {
            left: 15,
            top: 10,
            right: 26,
            bottom: 21
        })
    );
}

#[test]
fn the_window_is_cut_at_the_edges_of_the_frame() {
    assert_eq!(
        window(Point { x: 1.2, y: 28.9 }, 3.0, WIDTH, HEIGHT),
        Some(Window {
            left: 0,
            top: 19,
            right: 11,
            bottom: 30
        })
    );
    assert_eq!(window(Point { x: -10.5, y: 15.0 }, 3.0, WIDTH, HEIGHT), None);
    assert_eq!(window(Point { x: 20.0, y: 39.0 }, 3.0, WIDTH, HEIGHT), None);
}

#[test]
fn a_gray_darkened_below_black_stops_at_black_and_the_effective_contrast_says_so() {
    let mut pixels = plane(20);

    let effective = injected(&mut pixels, &target(0, vec![Point { x: 20.0, y: 15.0 }], -60.0, 3.0)).unwrap();

    assert_eq!(effective, -f64::from(gray(20)));
    assert_eq!(gray(pixels[15 * WIDTH + 20]), 0);
}

#[test]
fn the_effective_contrast_is_the_strongest_realized_change_in_the_direction_of_the_amplitude() {
    let center = Point { x: 20.0, y: 15.0 };
    for amplitude in [-40.0, 40.0] {
        let mut pixels = plane(BACKGROUND);

        let effective = injected(&mut pixels, &target(0, vec![center], amplitude, 3.0)).unwrap();

        let changes = pixels.iter().map(|&after| realized(BACKGROUND, after));
        let strongest = if amplitude < 0.0 { changes.min() } else { changes.max() };
        assert_eq!(effective, f64::from(strongest.unwrap()));
        assert!((effective - amplitude).abs() <= 1.0, "{amplitude}: {effective}");
    }
}

#[test]
fn a_target_out_of_the_frame_is_not_observed_and_changes_nothing() {
    let positions = vec![Point { x: -30.0, y: 15.0 }, Point { x: 20.0, y: 15.0 }];
    let mut injector = Injector::new(vec![target(0, positions, -40.0, 3.0)]);
    let mut pixels = plane(BACKGROUND);

    injector.inject(FrameIndex::new(0), LumaMut::new(&mut pixels, WIDTH).unwrap());

    assert!(pixels.iter().all(|&value| value == BACKGROUND));
    injector.inject(FrameIndex::new(1), LumaMut::new(&mut pixels, WIDTH).unwrap());
    injector.inject(FrameIndex::new(2), LumaMut::new(&mut pixels, WIDTH).unwrap());

    let finished = injector.finish();
    let observed = &finished[0].observations;
    assert_eq!(observed.len(), 1);
    assert_eq!(
        (observed[0].frame, observed[0].at),
        (FrameIndex::new(1), Point { x: 20.0, y: 15.0 })
    );
}

#[test]
fn two_targets_overlapping_in_a_frame_are_injected_one_after_the_other() {
    let first = target(0, vec![Point { x: 18.0, y: 15.0 }], -30.0, 3.0);
    let second = target(1, vec![Point { x: 22.0, y: 15.0 }], -30.0, 3.0);
    let mut one_by_one = plane(BACKGROUND);
    injected(&mut one_by_one, &first).unwrap();
    injected(&mut one_by_one, &second).unwrap();
    let mut injector = Injector::new(vec![first, second]);
    let mut pixels = plane(BACKGROUND);

    injector.inject(FrameIndex::new(0), LumaMut::new(&mut pixels, WIDTH).unwrap());

    assert_eq!(pixels, one_by_one);
    let finished = injector.finish();
    assert_eq!(
        finished.iter().map(|placed| placed.target.id()).collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(finished.iter().all(|placed| placed.observations.len() == 1));
    assert!(gray(pixels[15 * WIDTH + 20]) < gray(BACKGROUND) - 30);
}
