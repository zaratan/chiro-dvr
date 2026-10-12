use std::num::NonZeroU32;

use proptest::prelude::*;
use types::size::SourceSize;

use super::{Blur, Error, KERNEL_SUM, exact};
use crate::geometry::Geometry;

const BENCH_SIGMAS: [f64; 6] = [0.5, 0.75, 1.0, 1.5, 2.0, 3.0];
const SAFE_MARGIN: f64 = 1e-6;

fn work(width: u32, height: u32) -> Geometry {
    let source = SourceSize::new(
        NonZeroU32::new(width / 2 * 3).unwrap(),
        NonZeroU32::new(height / 2 * 3).unwrap(),
    );
    Geometry::new(source, NonZeroU32::new(width).unwrap()).unwrap()
}

fn kernel(sigma: f64) -> Vec<u32> {
    Blur::new(sigma, work(960, 720)).unwrap().weights().to_vec()
}

#[test]
fn one_work_pixel_gives_back_the_kernel_of_the_first_engine() {
    assert_eq!(kernel(1.0), [1, 18, 221, 991, 1634, 991, 221, 18, 1]);
}

#[test]
fn the_reach_is_four_sigmas_rounded_up_for_every_blur_of_the_bench() {
    let reaches = [0.5, 0.75, 1.0, 1.5, 2.0].map(|sigma| Blur::new(sigma, work(960, 720)).unwrap().reach());
    assert_eq!(reaches, [2, 3, 4, 6, 8]);
}

#[test]
fn every_kernel_of_the_bench_rounds_the_same_on_every_system() {
    for sigma in BENCH_SIGMAS {
        let shares = exact(sigma, kernel(sigma).len() / 2);
        let weights = kernel(sigma);
        let fraction = |share: f64| share - share.floor();
        assert!(
            shares
                .iter()
                .all(|&share| fraction(share).min(1.0 - fraction(share)) > SAFE_MARGIN),
            "{sigma}"
        );
        let (given, kept): (Vec<_>, Vec<_>) = shares
            .iter()
            .zip(&weights)
            .take(weights.len() / 2)
            .partition(|&(&share, &weight)| f64::from(weight) > share);
        let lowest_given = given.iter().map(|&(&share, _)| fraction(share)).fold(1.0, f64::min);
        let highest_kept = kept.iter().map(|&(&share, _)| fraction(share)).fold(0.0, f64::max);
        assert!(lowest_given - highest_kept > SAFE_MARGIN, "{sigma}");
    }
}

#[test]
fn each_weight_is_its_share_of_the_gaussian_within_one_part() {
    for sigma in BENCH_SIGMAS {
        let weights = kernel(sigma);
        for (&weight, share) in weights.iter().zip(exact(sigma, weights.len() / 2)) {
            assert!((f64::from(weight) - share).abs() < 1.0, "{sigma}: {weight} for {share}");
        }
    }
}

#[test]
fn every_kernel_of_the_bench_rises_to_its_centre() {
    for sigma in BENCH_SIGMAS {
        let weights = kernel(sigma);
        let centre = weights.len() / 2;
        assert!(
            weights[..=centre].windows(2).all(|pair| pair[0] <= pair[1]),
            "{sigma}: {weights:?}"
        );
    }
}

#[test]
fn no_blur_leaves_the_frame_as_it_is() {
    assert_eq!(kernel(0.0), [KERNEL_SUM]);
}

#[test]
fn a_blur_too_thin_to_spread_leaves_the_frame_as_it_is_rather_than_dividing_zero_by_zero() {
    for sigma in [f64::MIN_POSITIVE, 5e-324] {
        let weights = kernel(sigma);
        assert_eq!(weights.iter().sum::<u32>(), KERNEL_SUM);
        assert_eq!(weights[weights.len() / 2], KERNEL_SUM, "{sigma}");
    }
}

#[test]
fn a_work_frame_narrower_than_the_kernel_is_refused_before_the_kernel_is_built() {
    assert_eq!(
        Blur::new(3.0, work(24, 18)),
        Err(Error::TooSmall {
            width: 24,
            height: 18,
            kernel: 25
        })
    );
    assert!(Blur::new(1e300, work(24, 18)).is_err());
    assert_eq!(
        Blur::new(1e300, work(24, 18)).unwrap_err().to_string(),
        format!(
            "a 24×18 work frame is narrower than the {} pixels of the blur",
            u64::MAX
        )
    );
    assert_eq!(
        Blur::new(3.0, work(26, 18)).unwrap_err().to_string(),
        "a 26×18 work frame is narrower than the 25 pixels of the blur"
    );
    assert!(Blur::new(3.0, work(26, 26)).is_ok());
}

proptest! {
    #[test]
    fn every_kernel_is_symmetric_and_shares_exactly_4096_parts(sigma in 0.0..200.0_f64) {
        let weights = Blur::new(sigma, work(1700, 1700)).unwrap().weights().to_vec();
        prop_assert_eq!(weights.iter().sum::<u32>(), KERNEL_SUM);
        prop_assert!(weights.iter().eq(weights.iter().rev()));
    }
}
