use proptest::prelude::*;

use super::{Reduction, Rounding};
use crate::gray::GRAY;

fn reduced(luma: &[u8], source: (usize, usize), work: (usize, usize)) -> Vec<u8> {
    let mut gray = vec![0; work.0 * work.1];
    Reduction::between(source, work).run(luma, &mut gray);
    gray
}

fn overlap(work: usize, pixel: usize, source_length: usize, work_length: usize) -> u64 {
    let (start, end) = (work * source_length, (work + 1) * source_length);
    let (from, to) = (pixel * work_length, (pixel + 1) * work_length);
    to.min(end).saturating_sub(from.max(start)) as u64
}

fn by_area(luma: &[u8], (source_width, source_height): (usize, usize), (width, height): (usize, usize)) -> Vec<u8> {
    let divisor = (source_width * source_height) as u64;
    (0..width * height)
        .map(|work| {
            let (x, y) = (work % width, work / width);
            let total: u64 = (0..source_width * source_height)
                .map(|pixel| {
                    let (column, row) = (pixel % source_width, pixel / source_width);
                    overlap(x, column, source_width, width)
                        * overlap(y, row, source_height, height)
                        * u64::from(GRAY[usize::from(luma[pixel])])
                })
                .sum();
            ((total + divisor / 2) / divisor) as u8
        })
        .collect()
}

fn previous_three_to_two(luma: &[u8], source_width: usize) -> Vec<u8> {
    let width = source_width / 3 * 2;
    let halves = |three: &[u8]| {
        let [left, middle, right] = [0, 1, 2].map(|at| u16::from(GRAY[usize::from(three[at])]));
        [2 * left + middle, middle + 2 * right]
    };
    let rounded = |area: u16| ((area + 4) / 9) as u8;
    let mut gray = vec![0; luma.len() / 9 * 4];
    for (source, work) in luma
        .chunks_exact(3 * source_width)
        .zip(gray.chunks_exact_mut(2 * width))
    {
        let (top, bottom) = work.split_at_mut(width);
        for (pair, index) in (0..source_width).step_by(3).zip(0..) {
            let rows = [0, 1, 2].map(|row| halves(&source[row * source_width + pair..][..3]));
            top[2 * index] = rounded(2 * rows[0][0] + rows[1][0]);
            top[2 * index + 1] = rounded(2 * rows[0][1] + rows[1][1]);
            bottom[2 * index] = rounded(rows[1][0] + 2 * rows[2][0]);
            bottom[2 * index + 1] = rounded(rows[1][1] + 2 * rows[2][1]);
        }
    }
    gray
}

#[test]
fn three_to_two_weighs_each_source_pixel_four_two_two_or_one_ninths() {
    let near = [2, 1, 0];
    for (row, column) in (0..3).flat_map(|row| (0..3).map(move |column| (row, column))) {
        let mut luma = vec![0; 6 * 3];
        luma[row * 6 + column] = 255;

        let gray = reduced(&luma, (6, 3), (4, 2));

        let expected = |top: bool, left: bool| {
            let rows = if top { near[row] } else { near[2 - row] };
            let columns = if left { near[column] } else { near[2 - column] };
            ((255 * rows * columns + 4) / 9) as u8
        };
        assert_eq!(
            [gray[0], gray[1], gray[4], gray[5]],
            [
                expected(true, true),
                expected(true, false),
                expected(false, true),
                expected(false, false)
            ],
            "{row} {column}"
        );
        assert_eq!([gray[2], gray[3], gray[6], gray[7]], [0; 4]);
    }
}

#[test]
fn a_whole_ratio_averages_its_blocks_and_one_to_one_only_reads_the_gray() {
    let luma: Vec<u8> = (0..12).map(|pixel| 40 + 15 * pixel).collect();

    let halved = reduced(&luma, (4, 3), (2, 1));
    let same = reduced(&luma, (4, 3), (4, 3));

    let gray: Vec<u32> = luma.iter().map(|&value| u32::from(GRAY[usize::from(value)])).collect();
    let block = |columns: [usize; 2]| {
        let total: u32 = columns
            .iter()
            .map(|&column| gray[column] + gray[4 + column] + gray[8 + column])
            .sum();
        ((total + 3) / 6) as u8
    };
    assert_eq!(halved, [block([0, 1]), block([2, 3])]);
    assert_eq!(
        same,
        luma.iter().map(|&value| GRAY[usize::from(value)]).collect::<Vec<_>>()
    );
}

#[test]
fn a_frame_reduced_after_another_owes_nothing_to_the_rows_kept_from_it() {
    let first: Vec<u8> = (0..36 * 27).map(|pixel| (pixel % 200) as u8 + 30).collect();
    let second: Vec<u8> = (0..36 * 27).map(|pixel| 230 - (pixel % 190) as u8).collect();
    let mut reduction = Reduction::between((36, 27), (24, 18));
    let mut gray = vec![0; 24 * 18];

    reduction.run(&first, &mut gray);
    reduction.run(&second, &mut gray);

    assert_eq!(gray, reduced(&second, (36, 27), (24, 18)));
}

#[test]
fn a_symmetric_spot_keeps_its_centre_at_every_ratio() {
    let (source_width, source_height) = (180, 108);
    let (centre_x, centre_y) = (90.0, 53.0);
    let luma: Vec<u8> = (0..source_width * source_height)
        .map(|pixel| {
            let (x, y) = ((pixel % source_width) as f64, (pixel / source_width) as f64);
            let distance = ((x - centre_x).powi(2) + (y - centre_y).powi(2)).sqrt();
            if distance < 12.0 { 235 } else { 16 }
        })
        .collect();
    for width in [120, 90, 60, 125] {
        let height = (width as f64 * source_height as f64 / source_width as f64).round() as usize;
        let (scale_x, scale_y) = (source_width as f64 / width as f64, source_height as f64 / height as f64);

        let gray = reduced(&luma, (source_width, source_height), (width, height));

        let mass: f64 = gray.iter().map(|&value| f64::from(value)).sum();
        let (sum_x, sum_y) = gray.iter().enumerate().fold((0.0, 0.0), |(x, y), (pixel, &value)| {
            let weight = f64::from(value);
            (x + weight * (pixel % width) as f64, y + weight * (pixel / width) as f64)
        });
        let (x, y) = (
            (sum_x / mass + 0.5) * scale_x - 0.5,
            (sum_y / mass + 0.5) * scale_y - 0.5,
        );
        assert!(
            (x - centre_x).abs() < 0.1 && (y - centre_y).abs() < 0.1,
            "{width}: ({x}, {y})"
        );
    }
}

proptest! {
    #[test]
    fn every_work_pixel_is_the_rounded_area_weighted_gray_of_the_source_it_covers(
        (source_width, source_height, width, luma) in (1_usize..30, 1_usize..30)
            .prop_flat_map(|(w, h)| (Just(w), Just(h), 1..=w, prop::collection::vec(any::<u8>(), w * h))),
        rounding in 0.0..1.0_f64,
    ) {
        let height = ((width as f64 * source_height as f64 / source_width as f64) + rounding - 0.5)
            .round()
            .clamp(1.0, source_height as f64) as usize;

        let gray = reduced(&luma, (source_width, source_height), (width, height));

        prop_assert_eq!(gray, by_area(&luma, (source_width, source_height), (width, height)));
    }

    #[test]
    fn three_to_two_gives_back_the_reduction_of_the_first_engine_bit_for_bit(
        (thirds_wide, thirds_high, luma) in (1_usize..12, 1_usize..12)
            .prop_flat_map(|(w, h)| (Just(w), Just(h), prop::collection::vec(any::<u8>(), 9 * w * h))),
    ) {
        let (source_width, source_height) = (3 * thirds_wide, 3 * thirds_high);

        let gray = reduced(&luma, (source_width, source_height), (2 * thirds_wide, 2 * thirds_high));

        prop_assert_eq!(gray, previous_three_to_two(&luma, source_width));
    }
}

#[test]
fn a_divisor_too_large_for_thirty_two_bit_totals_falls_back_to_exact_division() {
    let divisor = 20_000_000;

    let rounding = Rounding::new(divisor);

    assert!(matches!(rounding, Rounding::Exact { .. }));
    assert_eq!(
        [0, divisor / 2 - 1, divisor / 2, 255 * divisor].map(|total| rounding.rounded(total)),
        [0, 0, 1, 255]
    );
}

const LARGEST_RECIPROCAL: u64 = 16_810_048;

#[test]
fn the_reciprocal_serves_every_divisor_from_two_whose_totals_fit_thirty_two_bits() {
    assert!(matches!(Rounding::new(2), Rounding::Reciprocal { .. }));
    assert!(matches!(Rounding::new(LARGEST_RECIPROCAL), Rounding::Reciprocal { .. }));
    assert!(matches!(Rounding::new(LARGEST_RECIPROCAL + 1), Rounding::Exact { .. }));
}

#[test]
fn one_to_one_keeps_every_total_through_exact_division() {
    let rounding = Rounding::new(1);

    assert!(matches!(rounding, Rounding::Exact { .. }));
    assert_eq!([0, 17, 255].map(|total| rounding.rounded(total)), [0, 17, 255]);
}

#[test]
fn the_reciprocal_rounds_exactly_on_every_boundary_where_the_level_changes() {
    for divisor in [2, 3, 9, 1296, 1441 * 1081, LARGEST_RECIPROCAL] {
        let rounding = Rounding::new(divisor);
        let half = divisor / 2;
        for level in 1..=255 {
            for total in [level * divisor - half - 1, level * divisor - half] {
                assert_eq!(
                    u64::from(rounding.rounded(total)),
                    (total + half) / divisor,
                    "{divisor} {total}"
                );
            }
        }
        assert_eq!(rounding.rounded(255 * divisor), 255);
    }
}

proptest! {
    #[test]
    fn dividing_by_the_reciprocal_rounds_exactly_like_a_division(
        (divisor, total) in (2_u64..=LARGEST_RECIPROCAL).prop_flat_map(|divisor| (Just(divisor), 0..=255 * divisor)),
    ) {
        let rounding = Rounding::new(divisor);

        let reciprocal = matches!(rounding, Rounding::Reciprocal { .. });
        prop_assert!(reciprocal);
        prop_assert_eq!(u64::from(rounding.rounded(total)), (total + divisor / 2) / divisor);
    }
}
