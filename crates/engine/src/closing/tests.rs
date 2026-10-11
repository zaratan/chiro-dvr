use std::collections::BTreeSet;
use std::num::NonZeroU32;

use proptest::prelude::*;
use types::size::SourceSize;

use super::{Closing, DISK, RADIUS};
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

fn inside_disk(row: isize, column: isize) -> bool {
    row.unsigned_abs() <= RADIUS && column.unsigned_abs() <= DISK[row.saturating_add_unsigned(RADIUS) as usize]
}

fn dense_closing(pixels: &BTreeSet<u32>) -> Vec<u32> {
    let reach = RADIUS as isize;
    let at = |row: isize, column: isize| -> Option<usize> {
        (0..HEIGHT as isize).contains(&row).then_some(())?;
        (0..WIDTH as isize)
            .contains(&column)
            .then(|| row as usize * WIDTH + column as usize)
    };
    let offsets: Vec<(isize, isize)> = (-reach..=reach)
        .flat_map(|row| (-reach..=reach).map(move |column| (row, column)))
        .filter(|&(row, column)| inside_disk(row, column))
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

#[test]
fn the_disk_is_the_ellipse_of_the_merge_radius_in_work_pixels() {
    let config = DetectConfig::default();
    let radius = (config.merge_radius() / geometry().scale().factor()).round();

    let halves: Vec<usize> = (-4..=4_i32)
        .map(|row| {
            let ratio = f64::from(row * row) / (radius * radius);
            (radius * (1.0 - ratio).sqrt()).round() as usize
        })
        .collect();

    assert_eq!(radius, RADIUS as f64);
    assert_eq!(halves, DISK);
}

#[test]
fn a_full_frame_flash_closes_to_the_whole_frame() {
    let every: BTreeSet<u32> = (0..(WIDTH * HEIGHT) as u32).collect();

    let closed = Closing::new(geometry()).run(&lit(&every)).to_vec();

    assert_eq!(closed, every.into_iter().collect::<Vec<_>>());
}

#[test]
fn a_lone_pixel_in_a_corner_stays_lit_because_the_outside_counts_as_lit() {
    let corner = BTreeSet::from([0]);

    assert_eq!(Closing::new(geometry()).run(&lit(&corner)), [0]);
}

#[test]
fn one_closing_after_another_starts_from_a_clean_frame() {
    let mut closing = Closing::new(geometry());
    let first = BTreeSet::from([30, 31, 40, 200]);
    let second = BTreeSet::from([100, 104]);

    closing.run(&lit(&first));
    let again = closing.run(&lit(&second)).to_vec();

    assert_eq!(again, dense_closing(&second));
}

proptest! {
    #[test]
    fn the_sparse_closing_matches_the_dense_one(pixels in prop::collection::btree_set(0..(WIDTH * HEIGHT) as u32, 0..60)) {
        let closed = Closing::new(geometry()).run(&lit(&pixels)).to_vec();

        prop_assert_eq!(closed, dense_closing(&pixels));
    }
}
