use std::num::NonZeroUsize;

use media::decode::{ActiveThreads, Threads};
use tempfile::TempDir;
use types::rational::Rational;

use super::{Error, detect};
use crate::config::DetectConfig;
use crate::input;
use crate::settings::DetectSettings;
use crate::synthetic::{luma, with_square};
use crate::testing::{self, FRAMES, binoculars};

const SOURCE: (u32, u32) = (360, 270);

fn spot(index: usize) -> Vec<u8> {
    with_square(vec![luma(200.0); 360 * 270], 360, (60 + 6 * index, 135), 9, -80.0)
}

fn three_to_two() -> DetectConfig {
    DetectConfig::try_from(&DetectSettings {
        work_width: 240,
        ..DetectSettings::default()
    })
    .unwrap()
}

#[test]
fn a_spot_crossing_a_binoculars_video_is_found_on_every_frame_without_damage() {
    let folder = TempDir::new().unwrap();
    let path = binoculars(
        &folder.path().join("spot.mp4"),
        SOURCE,
        Rational::new(30, 1).unwrap(),
        (0..20).map(spot),
    );

    let detected = detect(&path, Threads::Exactly(NonZeroUsize::MIN), &three_to_two()).unwrap();

    assert_eq!(detected.reading().frames().len(), 20);
    assert_eq!(detected.active_threads(), ActiveThreads::Single);
    assert_eq!(detected.detection_threads(), rayon::current_num_threads());
    assert_eq!(detected.half_window().get(), 15);
    assert_eq!(detected.damage().spans(), []);
    let background = detected.background().unwrap();
    assert_eq!((background.width(), background.height()), SOURCE);
    let level = |x: usize, y: usize| i32::from(background.y()[y * 360 + x]);
    assert!((level(64, 139) - level(300, 220)).abs() <= 2);
    for (frame, found) in detected.detections().iter() {
        assert_eq!(found.len(), 1, "{frame:?}: {found:?}");
        assert!(
            (found[0].x - (64.0 + 6.0 * frame.get() as f64)).abs() < 2.0,
            "{found:?}"
        );
        assert!((found[0].y - 139.0).abs() < 2.0, "{found:?}");
    }
}

#[test]
fn a_video_of_another_size_than_the_binoculars_is_read_and_detected_frame_by_frame() {
    let folder = TempDir::new().unwrap();
    let path = testing::encode(folder.path(), FRAMES);

    let detected = detect(&path, Threads::Automatic, &DetectConfig::default()).unwrap();

    assert_eq!(detected.detections().frame_count(), usize::from(FRAMES));
    assert_eq!(detected.reading().video().size().width().get(), testing::WIDTH);
}

#[test]
fn a_frame_rate_too_low_for_three_background_samples_is_refused() {
    let folder = TempDir::new().unwrap();
    let path = binoculars(
        &folder.path().join("slow.mp4"),
        SOURCE,
        Rational::new(2, 1).unwrap(),
        (0..3).map(spot),
    );

    let refused = detect(&path, Threads::Automatic, &DetectConfig::default()).unwrap_err();

    assert!(matches!(refused, Error::Config { .. }), "{refused:?}");
    assert_eq!(
        refused.to_string(),
        format!("cannot build the background window of {}", path.display())
    );
}

fn waits_for_the_generic_engine(error: &Error) -> Option<String> {
    error.unsupported().map(ToString::to_string)
}

#[test]
fn a_sample_count_the_engine_cannot_handle_yet_is_named_as_such() {
    let samples = Error::Config {
        path: std::path::PathBuf::from("video.mp4"),
        source: crate::config::Error::ManySamples {
            step: 1,
            frames: 31,
            samples: 31,
        },
    };

    assert!(waits_for_the_generic_engine(&samples).is_some_and(|reason| reason.contains("more than 16")));
}

#[test]
fn a_failure_the_generic_engine_will_not_lift_is_no_refusal() {
    let path = std::path::PathBuf::from("video.mp4");
    let few = Error::Config {
        path: path.clone(),
        source: crate::config::Error::FewSamples {
            step: 40,
            frames: 31,
            samples: 1,
        },
    };
    let small = Error::Blur {
        path: path.clone(),
        source: crate::blur::Error::TooSmall {
            width: 8,
            height: 6,
            kernel: 9,
        },
    };
    let silent = Error::Input(input::Error::NoAnnouncedFrames { path });
    for error in [few, small, silent] {
        assert_eq!(waits_for_the_generic_engine(&error), None, "{error}");
    }
}
