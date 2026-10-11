use std::num::NonZeroUsize;

use types::rate::FrameRate;
use types::rational::Rational;

use super::{DetectConfig, Error};

fn rate(num: i32, den: i32) -> FrameRate {
    FrameRate::new(Rational::new(num, den).unwrap()).unwrap()
}

fn stepping(step: usize) -> DetectConfig {
    DetectConfig {
        step: NonZeroUsize::new(step).unwrap(),
        ..DetectConfig::default()
    }
}

#[test]
fn the_whole_092_keeps_a_half_window_of_15_frames_sampled_one_in_three() {
    let half = DetectConfig::default()
        .half_window(rate(90_110_000, 3_000_927))
        .unwrap();

    assert_eq!(half.get(), 15);
    assert_eq!((2 * half.get() + 1).div_ceil(DetectConfig::default().step().get()), 11);
}

#[test]
fn background_window_too_short_for_the_frame_rate_is_rejected() {
    let short = DetectConfig {
        window_seconds: 0.01,
        ..DetectConfig::default()
    };

    assert!(matches!(
        short.half_window(rate(30, 1)),
        Err(Error::ShortWindow { frames: 1, .. })
    ));
}

#[test]
fn background_step_leaving_under_three_frames_is_rejected_rather_than_using_one_frame_as_background() {
    assert_eq!(
        stepping(40).half_window(rate(30, 1)),
        Err(Error::FewSamples {
            step: 40,
            frames: 31,
            samples: 1
        })
    );
}

#[test]
fn background_step_leaving_exactly_three_frames_is_accepted_and_one_more_is_rejected() {
    assert_eq!(stepping(15).half_window(rate(30, 1)).unwrap().get(), 15);
    assert!(matches!(
        stepping(16).half_window(rate(30, 1)),
        Err(Error::FewSamples { samples: 2, .. })
    ));
}

#[test]
fn more_than_sixteen_background_samples_wait_for_a_wider_median() {
    assert!(stepping(2).half_window(rate(30, 1)).is_ok());
    assert_eq!(
        stepping(1).half_window(rate(30, 1)),
        Err(Error::ManySamples {
            step: 1,
            frames: 31,
            samples: 31
        })
    );
}

#[test]
fn every_refusal_names_the_offending_numbers() {
    let message = stepping(40).half_window(rate(30, 1)).unwrap_err().to_string();

    assert_eq!(message, "one frame in 40 keeps 1 of 31 window frames, fewer than 3");
}

#[test]
fn defaults_are_those_of_the_reference_settings() {
    let config = DetectConfig::default();

    assert_eq!(
        (
            config.threshold(),
            config.min_area(),
            config.max_area(),
            config.window_seconds(),
            config.merge_radius(),
            config.noise_factor(),
            config.target_sigma(),
        ),
        (12.0, 4.0, 2700.0, 1.0, 6.0, 8.0, 1.5)
    );
    assert_eq!((config.step().get(), config.work_width().get()), (3, 960));
}
