use std::num::NonZeroU32;

use proptest::prelude::*;
use types::size::SourceSize;

use super::{Prepare, reflect, to_level, to_units};
use crate::blur::Blur;
use crate::geometry::Geometry;
use crate::gray::GRAY;
use crate::mask::Mask;
use crate::settings::Region;
use crate::synthetic::Noise;

const WIDTH: usize = 48;
const HEIGHT: usize = 36;

fn geometry() -> Geometry {
    let source = SourceSize::new(
        NonZeroU32::new(WIDTH as u32).unwrap(),
        NonZeroU32::new(HEIGHT as u32).unwrap(),
    );
    Geometry::new(source, NonZeroU32::new(32).unwrap()).unwrap()
}

fn unmasked(geometry: Geometry) -> Mask {
    Mask::new(&[], geometry.width(), geometry.height(), 0).unwrap()
}

fn prepared(luma: &[u8]) -> (Vec<u16>, u64) {
    prepared_under(luma, &unmasked(geometry()))
}

fn prepared_under(luma: &[u8], mask: &Mask) -> (Vec<u16>, u64) {
    let geometry = geometry();
    let mut out = vec![0; geometry.pixels()];
    let sum = Prepare::new(geometry, &Blur::new(1.0, geometry).unwrap()).run(luma, mask, &mut out);
    (out, sum)
}

#[test]
fn flat_luma_stays_flat_at_64_units_per_level_of_the_measured_gray() {
    let (out, sum) = prepared(&[126; WIDTH * HEIGHT]);

    let level = u16::from(GRAY[126]) * 64;
    assert!(out.iter().all(|&unit| unit == level));
    assert_eq!(sum, u64::from(level) * 32 * 24);
}

#[test]
fn the_brightest_luma_fits_in_sixteen_bits_and_the_darkest_stays_at_zero() {
    let (bright, _) = prepared(&[255; WIDTH * HEIGHT]);
    let (dark, sum) = prepared(&[0; WIDTH * HEIGHT]);

    assert!(bright.iter().all(|&unit| unit == 255 * 64));
    assert!(dark.iter().all(|&unit| unit == 0));
    assert_eq!(sum, 0);
}

#[test]
fn a_white_frame_stays_white_at_every_sigma_without_overflowing_the_folded_sums() {
    let geometry = geometry();
    for sigma in [0.0, 0.3, 1.0, 2.0, 2.75] {
        let blur = Blur::new(sigma, geometry).unwrap();
        let mut out = vec![0; geometry.pixels()];

        Prepare::new(geometry, &blur).run(&[255; WIDTH * HEIGHT], &unmasked(geometry), &mut out);

        assert!(out.iter().all(|&unit| unit == 255 * 64), "{sigma}");
    }
}

#[test]
fn a_checkerboard_of_extremes_averages_to_mid_gray_without_wrapping() {
    let luma: Vec<u8> = (0..WIDTH * HEIGHT)
        .map(|pixel| {
            if (pixel % WIDTH + pixel / WIDTH).is_multiple_of(2) {
                255
            } else {
                0
            }
        })
        .collect();

    let (out, sum) = prepared(&luma);

    let mean = sum as f64 / out.len() as f64;
    assert!((mean - 255.0 * 32.0).abs() < 0.01 * 255.0 * 64.0, "{mean}");
    assert!(out.iter().all(|&unit| (7_000..9_500).contains(&unit)), "{out:?}");
}

#[test]
fn the_filter_keeps_the_light_of_a_spot_away_from_the_borders() {
    let mut luma = vec![0; WIDTH * HEIGHT];
    for row in 15..18 {
        luma[row * WIDTH + 21..row * WIDTH + 24].fill(255);
    }

    let (out, sum) = prepared(&luma);

    let lit = out.iter().filter(|&&unit| unit > 0).count();
    assert!((sum as f64 - 4.0 * 255.0 * 64.0).abs() <= lit as f64 / 2.0);
    let peak = out.iter().position(|&unit| unit == *out.iter().max().unwrap()).unwrap();
    assert!([10 * 32 + 14, 10 * 32 + 15, 11 * 32 + 14, 11 * 32 + 15].contains(&peak));
}

#[test]
fn the_level_sum_leaves_out_the_masked_band_but_the_units_under_it_are_still_prepared() {
    let geometry = geometry();
    let luma: Vec<u8> = (0..geometry.source_width() * geometry.source_height())
        .map(|pixel| (pixel % 251) as u8)
        .collect();
    let band = Region {
        x0: 0.0,
        y0: 0.0,
        x1: 1.0,
        y1: 0.25,
    };
    let mask = Mask::new(&[band], geometry.width(), geometry.height(), 0).unwrap();
    let rows = band.rows(geometry.height(), 0).end;

    let (units, sum) = prepared(&luma);
    let (masked_units, masked_sum) = prepared_under(&luma, &mask);

    let band_sum: u64 = units[..rows * geometry.width()]
        .iter()
        .map(|&unit| u64::from(unit))
        .sum();
    assert_eq!(masked_units, units);
    assert_eq!(masked_sum, sum - band_sum);
}

#[test]
fn the_prepared_frame_below_the_masked_band_does_not_depend_on_what_the_display_shows() {
    let geometry = geometry();
    let blur = Blur::new(2.0, geometry).unwrap();
    let band = Region {
        x0: 0.0,
        y0: 0.0,
        x1: 1.0,
        y1: 8.0 / HEIGHT as f64,
    };
    let below = band.rows(geometry.height(), blur.reach()).end * geometry.width();
    let mut noise = Noise::new(17);
    let digits = |noise: &mut Noise| -> Vec<u8> {
        (0..WIDTH * HEIGHT)
            .map(|pixel| match (pixel < 8 * WIDTH, noise.uniform() < 0.5) {
                (false, _) => 120,
                (true, true) => 16,
                (true, false) => 235,
            })
            .collect()
    };
    let mut prepare = Prepare::new(geometry, &blur);
    let unmasked = unmasked(geometry);
    let (mut first, mut second) = (vec![0; geometry.pixels()], vec![0; geometry.pixels()]);

    prepare.run(&digits(&mut noise), &unmasked, &mut first);
    prepare.run(&digits(&mut noise), &unmasked, &mut second);

    assert_ne!(first[..below], second[..below]);
    assert_eq!(first[below..], second[below..]);
}

#[test]
fn borders_mirror_without_repeating_the_edge_pixel() {
    assert_eq!([0, 1, 2, 3].map(|shifted| reflect(shifted, 4, 10)), [4, 3, 2, 1]);
    assert_eq!([4, 13, 14, 17].map(|shifted| reflect(shifted, 4, 10)), [0, 9, 8, 5]);
    assert_eq!([0, 2, 8].map(|shifted| reflect(shifted, 2, 6)), [2, 0, 4]);
}

#[test]
fn thresholds_and_amplitudes_convert_between_gray_levels_and_work_units() {
    assert!((to_units(12.0) - 768.0).abs() < f32::EPSILON);
    assert!((to_level(800.0) - 12.5).abs() < f64::EPSILON);
}

fn mirrored(index: isize, length: isize) -> usize {
    let index = index.abs();
    (if index < length {
        index
    } else {
        2 * (length - 1) - index
    }) as usize
}

fn blurred_by_hand(luma: &[u8], width: usize, height: usize, weights: &[u32]) -> Vec<u16> {
    let reach = (weights.len() / 2) as isize;
    let gray: Vec<u64> = luma.iter().map(|&value| u64::from(GRAY[usize::from(value)])).collect();
    let across: Vec<u64> = (0..width * height)
        .map(|pixel| {
            let (x, y) = ((pixel % width) as isize, pixel / width);
            weights
                .iter()
                .enumerate()
                .map(|(tap, &weight)| {
                    u64::from(weight) * gray[y * width + mirrored(x + tap as isize - reach, width as isize)]
                })
                .sum()
        })
        .collect();
    let divisor = 4096 * 4096 / 64;
    (0..width * height)
        .map(|pixel| {
            let (x, y) = (pixel % width, (pixel / width) as isize);
            let total: u64 = weights
                .iter()
                .enumerate()
                .map(|(tap, &weight)| {
                    u64::from(weight) * across[mirrored(y + tap as isize - reach, height as isize) * width + x]
                })
                .sum();
            ((total + divisor / 2) / divisor) as u16
        })
        .collect()
}

proptest! {
    #[test]
    fn every_blur_gives_the_rounded_separable_convolution_with_mirrored_borders(
        (width, height, luma) in (10_usize..=40, 10_usize..=40)
            .prop_flat_map(|(w, h)| (Just(w), Just(h), prop::collection::vec(any::<u8>(), w * h))),
        sigma in 0.0..3.5_f64,
    ) {
        let source = SourceSize::new(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap());
        let geometry = Geometry::new(source, NonZeroU32::new(width as u32).unwrap()).unwrap();
        let blur = Blur::new(sigma, geometry);
        prop_assume!(blur.is_ok());
        let blur = blur.unwrap();
        let mut out = vec![0; width * height];

        Prepare::new(geometry, &blur).run(&luma, &unmasked(geometry), &mut out);

        prop_assert_eq!(out, blurred_by_hand(&luma, width, height, blur.weights()));
    }
}
