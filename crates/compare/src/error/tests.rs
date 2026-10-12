use std::path::PathBuf;

use engine::settings::{self, Bound, Setting, Unsupported};
use engine::{config, detection};

use super::Error;

fn reason(error: &Error) -> Option<String> {
    error.unsupported().map(ToString::to_string)
}

#[test]
fn settings_the_engine_cannot_apply_yet_are_a_refusal_named_by_its_setting() {
    let error = Error::Settings {
        video: "video.mp4".to_owned(),
        source: settings::Error::Unsupported(Unsupported::OsdRegions { count: 2 }),
    };
    assert_eq!(
        reason(&error).as_deref(),
        Some("osd_regions: masks are not handled, 2 given")
    );
}

#[test]
fn what_the_engine_cannot_detect_yet_is_a_refusal_as_the_engine_names_it() {
    let samples = Error::Detection(detection::Error::Config {
        path: PathBuf::from("video.mp4"),
        source: config::Error::ManySamples {
            step: 1,
            frames: 31,
            samples: 31,
        },
    });
    assert!(reason(&samples).is_some_and(|text| text.contains("more than 16")));
}

#[test]
fn invalid_settings_and_other_failures_are_failures_rather_than_refusals() {
    let invalid = Error::Settings {
        video: "video.mp4".to_owned(),
        source: settings::Error::Setting {
            setting: Setting::Threshold,
            value: -1.0,
            bound: Bound::Positive,
        },
    };
    let few = Error::Detection(detection::Error::Config {
        path: PathBuf::from("video.mp4"),
        source: config::Error::FewSamples {
            step: 40,
            frames: 31,
            samples: 1,
        },
    });
    let window = Error::HalfWindow {
        video: "video.mp4".to_owned(),
        expected: 15,
        found: 30,
    };
    for error in [invalid, few, window] {
        assert_eq!(reason(&error), None, "{error}");
    }
}
