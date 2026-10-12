use proptest::prelude::*;

use super::{Error, Mask};
use crate::settings::Region;

fn region(x0: f64, y0: f64, x1: f64, y1: f64) -> Region {
    Region { x0, y0, x1, y1 }
}

fn covered(mask: &Mask, width: usize, height: usize) -> Vec<usize> {
    (0..width * height).filter(|&pixel| mask.covers(pixel)).collect()
}

#[test]
fn a_region_is_rounded_outwards() {
    let mask = Mask::new(&[region(0.15, 0.15, 0.35, 0.35)], 10, 10, 0).unwrap();

    let expected: Vec<usize> = (1..4)
        .flat_map(|row| (1..4).map(move |column| row * 10 + column))
        .collect();
    assert_eq!(covered(&mask, 10, 10), expected);
    assert_eq!(mask.kept().get(), 91);
}

#[test]
fn no_region_masks_nothing() {
    let mask = Mask::new(&[], 10, 10, 3).unwrap();

    assert_eq!(covered(&mask, 10, 10), Vec::<usize>::new());
    assert_eq!(mask.kept().get(), 100);
    assert_eq!(mask.masked_sum(0, &[7; 10]), 0);
}

#[test]
fn regions_covering_the_whole_frame_are_refused() {
    let halves = [region(0.0, 0.0, 1.0, 0.5), region(0.0, 0.5, 1.0, 1.0)];

    let refused = Mask::new(&halves, 10, 10, 0).unwrap_err();

    assert_eq!(refused, Error::WholeFrame { width: 10, height: 10 });
    assert_eq!(refused.to_string(), "the OSD regions mask the whole 10×10 work frame");
}

#[test]
fn the_margin_grows_a_region_but_never_past_the_frame() {
    let mask = Mask::new(&[region(0.0, 0.0, 0.1, 0.1)], 10, 10, 5).unwrap();

    assert_eq!(mask.kept().get(), 100 - 36);
    assert!(mask.covers(5 * 10 + 5));
    assert!(!mask.covers(6 * 10));
}

#[test]
fn a_region_that_rounds_to_no_column_masks_nothing() {
    let mask = Mask::new(&[region(0.5, 0.0, 0.5, 1.0)], 10, 10, 0).unwrap();

    assert_eq!(covered(&mask, 10, 10), Vec::<usize>::new());
}

#[test]
fn overlapping_regions_are_counted_once() {
    let mask = Mask::new(&[region(0.0, 0.0, 0.5, 0.5), region(0.3, 0.0, 0.8, 0.5)], 10, 10, 0).unwrap();

    assert_eq!(mask.kept().get(), 100 - 5 * 8);
}

fn regions() -> impl Strategy<Value = Vec<Region>> {
    prop::collection::vec(
        (0.0_f64..1.0, 0.0_f64..1.0, 0.0_f64..1.0, 0.0_f64..1.0)
            .prop_map(|(a, b, c, d)| region(a.min(b), c.min(d), a.max(b), c.max(d))),
        1..4,
    )
}

proptest! {
    #[test]
    fn the_masked_sum_of_a_row_is_the_sum_of_its_covered_pixels(
        regions in regions(),
        width in 1_usize..40,
        height in 1_usize..40,
        margin in 0_usize..5,
        seed in any::<u16>(),
    ) {
        let line: Vec<u16> = (0..width).map(|column| seed.wrapping_mul(31).wrapping_add(column as u16)).collect();
        match Mask::new(&regions, width, height, margin) {
            Ok(mask) => {
                for pixel in 0..width * height {
                    let (row, column) = (pixel / width, pixel % width);
                    let in_a_region = regions.iter().any(|region| {
                        region.rows(height, margin).contains(&row) && region.columns(width, margin).contains(&column)
                    });
                    prop_assert_eq!(mask.covers(pixel), in_a_region);
                }
                for row in 0..height {
                    let by_pixel: u64 = (0..width)
                        .filter(|column| mask.covers(row * width + column))
                        .map(|column| u64::from(line[column]))
                        .sum();
                    prop_assert_eq!(mask.masked_sum(row, &line), by_pixel);
                }
                prop_assert_eq!(mask.kept().get(), width * height - covered(&mask, width, height).len());
            }
            Err(Error::WholeFrame { .. }) => prop_assert!(covered_by_regions(&regions, width, height, margin)),
        }
    }
}

fn covered_by_regions(regions: &[Region], width: usize, height: usize, margin: usize) -> bool {
    (0..height).all(|row| {
        (0..width).all(|column| {
            regions.iter().any(|region| {
                region.rows(height, margin).contains(&row) && region.columns(width, margin).contains(&column)
            })
        })
    })
}
