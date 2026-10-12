use std::collections::BTreeSet;
use std::num::NonZeroU32;

use proptest::prelude::*;
use types::size::SourceSize;

use super::{Closing, Disk};
use crate::config::DetectConfig;
use crate::geometry::Geometry;
use crate::threshold::Lit;

const WIDTH: usize = 24;
const HEIGHT: usize = 18;

fn geometry() -> Geometry {
    let source = SourceSize::new(NonZeroU32::new(36).unwrap(), NonZeroU32::new(27).unwrap());
    Geometry::new(source, NonZeroU32::new(24).unwrap()).unwrap()
}

fn lit(pixels: &BTreeSet<u32>) -> Vec<Lit> {
    pixels.iter().map(|&pixel| Lit { pixel, magnitude: 1.0 }).collect()
}

fn inside_disk(radius: usize, row: isize, column: isize) -> bool {
    let (radius, row) = (radius as f64, row.unsigned_abs() as f64);
    row <= radius && column.unsigned_abs() as f64 <= (radius * radius - row * row).sqrt().round()
}

fn dense_closing_by(radius: usize, pixels: &BTreeSet<u32>) -> Vec<u32> {
    let reach = radius as isize;
    let at = |row: isize, column: isize| -> Option<usize> {
        (0..HEIGHT as isize).contains(&row).then_some(())?;
        (0..WIDTH as isize)
            .contains(&column)
            .then(|| row as usize * WIDTH + column as usize)
    };
    let offsets: Vec<(isize, isize)> = (-reach..=reach)
        .flat_map(|row| (-reach..=reach).map(move |column| (row, column)))
        .filter(|&(row, column)| inside_disk(radius, row, column))
        .collect();
    let dilated: Vec<bool> = (0..WIDTH * HEIGHT)
        .map(|pixel| {
            let (row, column) = ((pixel / WIDTH) as isize, (pixel % WIDTH) as isize);
            offsets.iter().any(|&(down, right)| {
                at(row + down, column + right).is_some_and(|around| pixels.contains(&(around as u32)))
            })
        })
        .collect();
    (0..WIDTH * HEIGHT)
        .filter(|&pixel| {
            let (row, column) = ((pixel / WIDTH) as isize, (pixel % WIDTH) as isize);
            offsets
                .iter()
                .all(|&(down, right)| at(row + down, column + right).is_none_or(|around| dilated[around]))
        })
        .map(|pixel| pixel as u32)
        .collect()
}

fn dense_closing(pixels: &BTreeSet<u32>) -> Vec<u32> {
    dense_closing_by(4, pixels)
}

fn closing(radius: f64) -> Closing {
    Closing::new(geometry(), &Disk::new(radius, geometry()))
}

fn wide() -> Geometry {
    let source = SourceSize::new(NonZeroU32::new(96).unwrap(), NonZeroU32::new(96).unwrap());
    Geometry::new(source, NonZeroU32::new(64).unwrap()).unwrap()
}

#[test]
fn the_default_merge_radius_gives_back_the_disk_of_the_first_engine() {
    let config = DetectConfig::default();
    let disk = Disk::new(config.merge_radius() / 1.5, wide());

    assert_eq!(disk.halves, [0, 3, 3, 4, 4, 4, 3, 3, 0]);
}

#[test]
fn each_row_of_the_disk_is_the_ellipse_rounded_to_whole_pixels() {
    for radius in 0..=8_usize {
        let disk = Disk::new(radius as f64, wide());

        let halves: Vec<usize> = (0..=2 * radius)
            .map(|row| {
                let offset = row.abs_diff(radius) as f64;
                ((radius * radius) as f64 - offset * offset).sqrt().round() as usize
            })
            .collect();

        assert_eq!(disk.halves, halves, "{radius}");
    }
}

#[test]
fn a_radius_half_way_between_two_rounds_up() {
    assert_eq!(Disk::new(2.5, wide()).halves.len(), 7);
}

#[test]
fn a_radius_of_zero_closes_nothing() {
    let pixels = BTreeSet::from([0, 5, 6, 30, 200, 431]);

    let closed = closing(0.0).run(&lit(&pixels)).to_vec();

    assert_eq!(closed, pixels.into_iter().collect::<Vec<_>>());
}

#[test]
fn a_radius_wider_than_the_frame_keeps_only_what_can_land_in_it_and_closes_the_same() {
    let pixels = BTreeSet::from([0, 13, 40, 250, 431]);
    let disk = Disk::new(1e12, geometry());

    let closed = Closing::new(geometry(), &disk).run(&lit(&pixels)).to_vec();

    assert_eq!(disk.halves.len(), 2 * (HEIGHT - 1) + 1);
    assert!(disk.halves.iter().all(|&half| half < WIDTH));
    assert_eq!(closed, dense_closing_by(60, &pixels));
}

#[test]
fn a_full_frame_flash_closes_to_the_whole_frame() {
    let every: BTreeSet<u32> = (0..(WIDTH * HEIGHT) as u32).collect();

    let closed = closing(4.0).run(&lit(&every)).to_vec();

    assert_eq!(closed, every.into_iter().collect::<Vec<_>>());
}

#[test]
fn a_lone_pixel_in_a_corner_stays_lit_because_the_outside_counts_as_lit() {
    let corner = BTreeSet::from([0]);

    assert_eq!(closing(4.0).run(&lit(&corner)), [0]);
}

#[test]
fn one_closing_after_another_starts_from_a_clean_frame() {
    let mut closing = closing(4.0);
    let first = BTreeSet::from([30, 31, 40, 200]);
    let second = BTreeSet::from([100, 104]);

    closing.run(&lit(&first));
    let again = closing.run(&lit(&second)).to_vec();

    assert_eq!(again, dense_closing(&second));
}

proptest! {
    #[test]
    fn the_sparse_closing_matches_the_dense_one_at_every_radius(
        pixels in prop::collection::btree_set(0..(WIDTH * HEIGHT) as u32, 0..60),
        radius in 0..=6_usize,
    ) {
        let closed = closing(radius as f64).run(&lit(&pixels)).to_vec();

        prop_assert_eq!(closed, dense_closing_by(radius, &pixels));
    }
}
