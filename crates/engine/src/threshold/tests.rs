use std::num::NonZeroUsize;

use super::{Gains, Lit, MIN_NOISE_SAMPLES, Threshold, median};
use crate::config::DetectConfig;
use crate::network::Network;
use crate::synthetic::Noise;

const PIXELS: usize = 2000;
const LEVEL: f64 = 120.0 * 64.0;

fn samples(count: usize, sigma: f64, seed: u64) -> Vec<Vec<u16>> {
    let mut noise = Noise::new(seed);
    (0..count)
        .map(|_| {
            (0..PIXELS)
                .map(|_| (LEVEL + sigma * 64.0 * noise.gaussian()).round() as u16)
                .collect()
        })
        .collect()
}

fn gains_of(samples: &[Vec<u16>]) -> Vec<f32> {
    let levels: Vec<f64> = samples
        .iter()
        .map(|sample| sample.iter().map(|&unit| f64::from(unit)).sum::<f64>() / PIXELS as f64)
        .collect();
    let mean = levels.iter().sum::<f64>() / levels.len() as f64;
    levels.iter().map(|level| (level - mean) as f32).collect()
}

fn lit(samples: &[Vec<u16>], current: usize, gains: &[f32]) -> Vec<Lit> {
    let views: Vec<&[u16]> = samples.iter().map(Vec::as_slice).collect();
    let network = Network::median(NonZeroUsize::new(samples.len()).unwrap());
    let mut lit = Vec::new();
    Threshold::new(&DetectConfig::default()).lit(
        &views,
        &samples[current],
        &network,
        Gains {
            current: gains[current],
            samples: gains,
        },
        &mut lit,
    );
    lit
}

fn lit_one_by_one(samples: &[Vec<u16>], current: usize, gains: &[f32]) -> Vec<Lit> {
    let threshold = Threshold::new(&DetectConfig::default());
    (0..PIXELS)
        .filter_map(|pixel| {
            let mut column: Vec<f32> = samples.iter().map(|sample| f32::from(sample[pixel])).collect();
            let background = median(&mut column);
            let magnitude = (f32::from(samples[current][pixel]) - background - gains[current]).abs();
            let mut deviations: Vec<f32> = samples
                .iter()
                .zip(gains)
                .map(|(sample, gain)| f32::from(sample[pixel]) - background - gain)
                .collect();
            let center = median(&mut deviations);
            let mut spread: Vec<f32> = deviations.iter().map(|deviation| (deviation - center).abs()).collect();
            let raised = threshold.floor.max(threshold.noise_factor * median(&mut spread));
            let bar = if samples.len() >= MIN_NOISE_SAMPLES {
                raised
            } else {
                threshold.floor
            };
            (magnitude > bar).then_some(Lit {
                pixel: pixel as u32,
                magnitude,
            })
        })
        .collect()
}

#[test]
fn pixels_are_lit_above_the_floor_and_eight_scaled_median_absolute_deviations_one_by_one() {
    for count in [3, 6, 7, 8, 11, 16] {
        let samples = samples(count, 6.0, count as u64);
        let gains = gains_of(&samples);

        let found = lit(&samples, count / 2, &gains);

        assert_eq!(found, lit_one_by_one(&samples, count / 2, &gains), "{count}");
        assert!(!found.is_empty() || count >= MIN_NOISE_SAMPLES, "{count}");
    }
}

#[test]
fn the_raised_threshold_follows_the_noise_of_each_pixel() {
    let samples = samples(11, 6.0, 1);
    let gains = gains_of(&samples);
    let views: Vec<&[u16]> = samples.iter().map(Vec::as_slice).collect();
    let threshold = Threshold::new(&DetectConfig::default());

    let mut raised: Vec<f32> = (0..PIXELS)
        .map(|pixel| threshold.raised(&views, pixel, LEVEL as f32, &gains) / 64.0)
        .collect();

    let typical = median(&mut raised);
    assert!((typical - 8.0 * 6.0).abs() < 0.25 * 8.0 * 6.0, "{typical}");
}

#[test]
fn a_global_gain_jump_does_not_inflate_the_noise() {
    let calm = samples(11, 3.0, 2);
    let jumped: Vec<Vec<u16>> = calm
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let shift = if index < 5 { 0 } else { 30 * 64 };
            sample.iter().map(|unit| unit + shift).collect()
        })
        .collect();

    let (steady, after_jump) = (typical_raise(&calm), typical_raise(&jumped));

    assert!((after_jump - steady).abs() < 8.0, "{after_jump} {steady}");
}

fn typical_raise(samples: &[Vec<u16>]) -> f32 {
    let threshold = Threshold::new(&DetectConfig::default());
    let gains = gains_of(samples);
    let views: Vec<&[u16]> = samples.iter().map(Vec::as_slice).collect();
    let mut raised: Vec<f32> = (0..PIXELS)
        .map(|pixel| {
            let mut column: Vec<f32> = samples.iter().map(|sample| f32::from(sample[pixel])).collect();
            threshold.raised(&views, pixel, median(&mut column), &gains)
        })
        .collect();
    median(&mut raised) / 64.0
}

#[test]
fn a_residual_below_the_floor_everywhere_lights_nothing() {
    let flat = vec![vec![120 * 64; PIXELS]; 11];
    let mut current = flat.clone();
    current[5] = vec![123 * 64; PIXELS];

    assert_eq!(lit(&current, 5, &[0.0; 11]), []);
}

#[test]
fn a_short_window_at_the_video_edges_keeps_the_fixed_floor() {
    let mut six = samples(6, 6.0, 3);
    for (sample, gray) in six.iter_mut().zip([120, 100, 120, 140, 120, 140]) {
        sample[0] = gray * 64;
    }
    let mut seven = six.clone();
    seven.push(six[1].clone());

    assert!(lit(&six, 3, &[0.0; 6]).iter().any(|lit| lit.pixel == 0));
    assert!(!lit(&seven, 3, &[0.0; 7]).iter().any(|lit| lit.pixel == 0));
}

#[test]
fn an_even_count_takes_the_mean_of_the_two_middle_values() {
    assert!((median(&mut [4.0, 1.0, 3.0, 2.0]) - 2.5).abs() < f32::EPSILON);
}

#[test]
fn a_residual_exactly_at_the_floor_lights_nothing() {
    let mut three = vec![vec![120 * 64; PIXELS]; 3];
    three[1] = vec![132 * 64; PIXELS];

    assert_eq!(lit(&three, 1, &[0.0; 3]), []);
}
