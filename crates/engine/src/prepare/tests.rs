use std::num::NonZeroU32;

use types::size::SourceSize;

use super::{GRAY, KERNEL, Prepare, reduce, reflect, to_level, to_units};
use crate::config::DetectConfig;
use crate::geometry::Geometry;

const WIDTH: usize = 48;
const HEIGHT: usize = 36;

fn geometry() -> Geometry {
    let source = SourceSize::new(
        NonZeroU32::new(WIDTH as u32).unwrap(),
        NonZeroU32::new(HEIGHT as u32).unwrap(),
    );
    Geometry::new(source, NonZeroU32::new(32).unwrap()).unwrap()
}

fn prepared(luma: &[u8]) -> (Vec<u16>, u64) {
    let geometry = geometry();
    let mut out = vec![0; geometry.pixels()];
    let sum = Prepare::new(geometry).run(luma, &mut out);
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
fn the_kernel_is_the_gaussian_of_one_work_pixel_quantised_to_4096() {
    let config = DetectConfig::default();
    let sigma = config.target_sigma() / geometry().scale().factor();
    let weights: Vec<f64> = (-4..=4_i32)
        .map(|offset| (-f64::from(offset * offset) / (2.0 * sigma * sigma)).exp())
        .collect();
    let total: f64 = weights.iter().sum();

    let quantised: Vec<u32> = weights
        .iter()
        .map(|weight| (weight / total * 4096.0).round() as u32)
        .collect();

    assert_eq!(quantised, KERNEL);
    assert_eq!(KERNEL.iter().sum::<u32>(), 4096);
}

#[test]
fn borders_mirror_without_repeating_the_edge_pixel() {
    assert_eq!([0, 1, 2, 3].map(|shifted| reflect(shifted, 10)), [4, 3, 2, 1]);
    assert_eq!([4, 13, 14, 17].map(|shifted| reflect(shifted, 10)), [0, 9, 8, 5]);
}

#[test]
fn thresholds_and_amplitudes_convert_between_gray_levels_and_work_units() {
    assert!((to_units(12.0) - 768.0).abs() < f32::EPSILON);
    assert!((to_level(800.0) - 12.5).abs() < f64::EPSILON);
}

#[test]
fn the_measured_gray_follows_the_limited_range_one_to_two_and_a_half_levels_below_its_formula() {
    let below: Vec<f64> = (17..=236_u8)
        .map(|luma| (f64::from(luma) - 16.0) * 255.0 / 219.0 - f64::from(GRAY[usize::from(luma)]))
        .collect();

    assert!(below.iter().all(|&gap| (1.1..2.4).contains(&gap)), "{below:?}");
    assert!(GRAY.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!((GRAY[17], GRAY[18], GRAY[236], GRAY[237]), (0, 1, 254, 255));
}

#[test]
fn the_reduction_weighs_each_source_pixel_by_its_overlap_with_each_work_pixel() {
    let near = [2, 1, 0];
    for (row, column) in (0..3).flat_map(|row| (0..3).map(move |column| (row, column))) {
        let mut luma = vec![0; 6 * 3];
        luma[row * 6 + column] = 255;
        let mut gray = vec![0; 4 * 2];

        reduce(&luma, 6, 4, &mut gray);

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
