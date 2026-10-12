use std::num::NonZeroUsize;

use proptest::prelude::*;
use types::rate::FrameRate;
use types::rational::Rational;

use super::{DetectConfig, Error};
use crate::settings::{self, DetectSettings, Region, Setting};
use crate::window::most_sampled;

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
fn any_number_of_background_samples_is_taken() {
    let long = converted(&DetectSettings {
        bg_window_s: 2.0,
        bg_step: 1,
        ..DetectSettings::default()
    })
    .unwrap();
    let half = long.half_window(rate(30, 1)).unwrap();

    assert_eq!(stepping(1).half_window(rate(30, 1)).unwrap().get(), 15);
    assert_eq!(most_sampled(half, long.step()).get(), 61);
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

fn converted(settings: &DetectSettings) -> Result<DetectConfig, settings::Error> {
    DetectConfig::try_from(settings)
}

fn out_of_bounds(settings: &DetectSettings) -> (Setting, f64) {
    match converted(settings) {
        Err(settings::Error::Setting { setting, value, .. }) => (setting, value),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_default_settings_give_the_default_configuration() {
    assert_eq!(converted(&DetectSettings::default()), Ok(DetectConfig::default()));
}

#[test]
fn the_settings_the_engine_handles_reach_the_configuration() {
    let config = converted(&DetectSettings {
        threshold: 25.0,
        min_area: 9.0,
        max_area: 500.0,
        bg_window_s: 2.0,
        bg_step: 1,
        work_width: 480,
        noise_factor: 0.0,
        ..DetectSettings::default()
    })
    .unwrap();
    assert_eq!(
        (
            config.threshold(),
            config.min_area(),
            config.max_area(),
            config.noise_factor()
        ),
        (25.0, 9.0, 500.0, 0.0)
    );
    assert_eq!(
        (config.window_seconds(), config.step().get(), config.work_width().get()),
        (2.0, 1, 480)
    );
}

#[test]
fn a_threshold_or_an_area_that_is_not_a_positive_number_is_refused() {
    for value in [0.0, -1.0, f64::INFINITY] {
        assert_eq!(
            out_of_bounds(&DetectSettings {
                threshold: value,
                ..DetectSettings::default()
            }),
            (Setting::Threshold, value)
        );
    }
    let nan = out_of_bounds(&DetectSettings {
        min_area: f64::NAN,
        ..DetectSettings::default()
    });
    assert_eq!(nan.0, Setting::MinArea);
    assert_eq!(
        out_of_bounds(&DetectSettings {
            max_area: 0.0,
            ..DetectSettings::default()
        }),
        (Setting::MaxArea, 0.0)
    );
}

#[test]
fn a_smallest_area_above_the_largest_is_refused() {
    let error = converted(&DetectSettings {
        min_area: 600.0,
        max_area: 500.0,
        ..DetectSettings::default()
    })
    .unwrap_err();
    assert_eq!(
        error,
        settings::Error::Areas {
            min_area: 600.0,
            max_area: 500.0
        }
    );
    assert_eq!(error.to_string(), "min_area 600 is larger than max_area 500");
}

#[test]
fn equal_smallest_and_largest_areas_are_accepted() {
    assert!(
        converted(&DetectSettings {
            min_area: 9.0,
            max_area: 9.0,
            ..DetectSettings::default()
        })
        .is_ok()
    );
}

#[test]
fn a_window_a_step_or_a_work_width_the_v011_refuses_is_refused() {
    assert_eq!(
        out_of_bounds(&DetectSettings {
            bg_window_s: 0.0,
            ..DetectSettings::default()
        }),
        (Setting::BgWindowS, 0.0)
    );
    assert_eq!(
        out_of_bounds(&DetectSettings {
            bg_step: 0,
            ..DetectSettings::default()
        }),
        (Setting::BgStep, 0.0)
    );
    assert_eq!(
        out_of_bounds(&DetectSettings {
            work_width: 15,
            ..DetectSettings::default()
        }),
        (Setting::WorkWidth, 15.0)
    );
    assert!(
        converted(&DetectSettings {
            work_width: 16,
            ..DetectSettings::default()
        })
        .is_ok()
    );
}

#[test]
fn a_negative_or_endless_radius_noise_factor_or_blur_is_refused() {
    assert_eq!(
        out_of_bounds(&DetectSettings {
            merge_radius: -1.0,
            ..DetectSettings::default()
        }),
        (Setting::MergeRadius, -1.0)
    );
    assert_eq!(
        out_of_bounds(&DetectSettings {
            noise_factor: f64::INFINITY,
            ..DetectSettings::default()
        }),
        (Setting::NoiseFactor, f64::INFINITY)
    );
    assert_eq!(
        out_of_bounds(&DetectSettings {
            target_sigma: -0.5,
            ..DetectSettings::default()
        }),
        (Setting::TargetSigma, -0.5)
    );
}

#[test]
fn an_osd_region_outside_the_frame_or_empty_is_refused_by_its_index() {
    let inside = Region {
        x0: 0.0,
        y0: 0.0,
        x1: 1.0,
        y1: 0.07,
    };
    for wrong in [
        Region { x1: 1.5, ..inside },
        Region { x1: 0.0, ..inside },
        Region { y0: 0.07, ..inside },
        Region { x0: f64::NAN, ..inside },
    ] {
        assert!(matches!(
            converted(&DetectSettings {
                osd_regions: vec![inside, wrong],
                ..DetectSettings::default()
            }),
            Err(settings::Error::Region { index: 1, .. })
        ));
    }
}

#[test]
fn any_blur_closing_radius_and_osd_region_reach_the_configuration() {
    let band = Region {
        x0: 0.0,
        y0: 0.0,
        x1: 1.0,
        y1: 0.07,
    };
    let taken = converted(&DetectSettings {
        target_sigma: 0.0,
        merge_radius: 0.0,
        osd_regions: vec![band],
        ..DetectSettings::default()
    })
    .unwrap();

    assert_eq!((taken.target_sigma(), taken.merge_radius()), (0.0, 0.0));
    assert_eq!(taken.osd_regions(), [band]);
}

#[test]
fn an_osd_region_may_reach_every_edge_of_the_frame() {
    let whole = Region {
        x0: 0.0,
        y0: 0.0,
        x1: 1.0,
        y1: 1.0,
    };

    let taken = converted(&DetectSettings {
        osd_regions: vec![whole],
        ..DetectSettings::default()
    })
    .unwrap();

    assert_eq!(taken.osd_regions(), [whole]);
}

#[test]
fn a_closing_radius_up_to_12_source_pixels_is_taken_and_a_wider_one_is_refused_to_keep_a_flash_frame_under_a_second() {
    let widest = converted(&DetectSettings {
        merge_radius: 12.0,
        ..DetectSettings::default()
    });
    let refused = [12.5, f64::NAN].map(|merge_radius| {
        converted(&DetectSettings {
            merge_radius,
            ..DetectSettings::default()
        })
        .unwrap_err()
        .to_string()
    });

    assert_eq!(widest.unwrap().merge_radius(), 12.0);
    assert_eq!(
        refused,
        [
            "merge_radius must be a finite number between 0 and 12, not 12.5",
            "merge_radius must be a finite number between 0 and 12, not NaN"
        ]
    );
}

fn region() -> impl Strategy<Value = Region> {
    (0.0_f64..0.5, 0.0_f64..0.5, 0.5_f64..=1.0, 0.5_f64..=1.0).prop_map(|(x0, y0, x1, y1)| Region { x0, y0, x1, y1 })
}

proptest! {
    #[test]
    fn every_valid_setting_is_written_back_as_it_was_read(
        threshold in 0.1_f64..100.0,
        min_area in 0.1_f64..10.0,
        max_area in 10.0_f64..5000.0,
        bg_window_s in 0.1_f64..5.0,
        bg_step in 1_usize..10,
        merge_radius in 0.0_f64..=12.0,
        work_width in 16_u32..3000,
        noise_factor in 0.0_f64..20.0,
        target_sigma in 0.0_f64..5.0,
        osd_regions in prop::collection::vec(region(), 0..4),
    ) {
        let settings = DetectSettings {
            threshold,
            min_area,
            max_area,
            bg_window_s,
            bg_step,
            osd_regions,
            merge_radius,
            work_width,
            noise_factor,
            target_sigma,
        };

        prop_assert_eq!(DetectSettings::from(&converted(&settings).unwrap()), settings);
    }
}

#[test]
fn a_refused_setting_says_which_bound_it_breaks() {
    let messages = [
        DetectSettings {
            work_width: 15,
            ..DetectSettings::default()
        },
        DetectSettings {
            threshold: 0.0,
            ..DetectSettings::default()
        },
        DetectSettings {
            noise_factor: -1.0,
            ..DetectSettings::default()
        },
    ]
    .map(|settings| converted(&settings).unwrap_err().to_string());
    assert_eq!(
        messages,
        [
            "work_width must be an integer ≥ 16, not 15",
            "threshold must be a finite number > 0, not 0",
            "noise_factor must be a finite number ≥ 0, not -1"
        ]
    );
}
