use std::num::NonZeroUsize;

use media::decode::{ActiveThreads, Threads};
use tempfile::TempDir;
use types::rational::Rational;

use super::{Error, detect};
use crate::config::DetectConfig;
use crate::input;
use crate::synthetic::{luma, with_square};
use crate::testing::{self, FRAMES, binoculars};

fn spot(index: usize) -> Vec<u8> {
    with_square(vec![luma(200.0); 1440 * 1080], 1440, (300 + 12 * index, 600), 9, -80.0)
}

#[test]
fn a_spot_crossing_a_binoculars_video_is_found_on_every_frame_without_damage() {
    let folder = TempDir::new().unwrap();
    let path = binoculars(
        &folder.path().join("spot.mp4"),
        Rational::new(30, 1).unwrap(),
        (0..20).map(spot),
    );

    let detected = detect(&path, Threads::Exactly(NonZeroUsize::MIN), &DetectConfig::default()).unwrap();

    assert_eq!(detected.reading().frames().len(), 20);
    assert_eq!(detected.active_threads(), ActiveThreads::Single);
    assert_eq!(detected.detection_threads(), rayon::current_num_threads());
    assert_eq!(detected.half_window().get(), 15);
    assert_eq!(detected.damage().spans(), []);
    let summary: Vec<usize> = detected.summary_frames().iter().map(|(frame, _)| frame.get()).collect();
    assert_eq!(summary, (0..20).collect::<Vec<_>>());
    assert!(
        detected
            .summary_frames()
            .iter()
            .all(|(_, yuv)| (yuv.width(), yuv.height()) == (1440, 1080))
    );
    for (frame, found) in detected.detections().iter() {
        assert_eq!(found.len(), 1, "{frame:?}: {found:?}");
        assert!(
            (found[0].x - (304.0 + 12.0 * frame.get() as f64)).abs() < 2.0,
            "{found:?}"
        );
        assert!((found[0].y - 604.0).abs() < 2.0, "{found:?}");
    }
}

#[test]
fn a_video_of_another_size_is_refused_before_any_detection() {
    let folder = TempDir::new().unwrap();
    let path = testing::encode(folder.path(), FRAMES);

    let refused = detect(&path, Threads::Automatic, &DetectConfig::default());

    assert!(matches!(
        refused,
        Err(Error::Input(input::Error::UnsupportedSize { .. }))
    ));
}

#[test]
fn a_frame_rate_too_low_for_three_background_samples_is_refused() {
    let folder = TempDir::new().unwrap();
    let path = binoculars(
        &folder.path().join("slow.mp4"),
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
fn a_size_a_range_a_reduction_or_a_sample_count_the_engine_cannot_handle_yet_is_named_as_such() {
    let path = std::path::PathBuf::from("video.mp4");
    let size = Error::Input(input::Error::UnsupportedSize {
        path: path.clone(),
        width: 720,
        height: 540,
    });
    let range = Error::Input(input::Error::FullRange { path: path.clone() });
    let reduction = Error::Geometry {
        path: path.clone(),
        source: crate::geometry::Error::Reduction {
            width: 1440,
            height: 1080,
            work_width: 480,
        },
    };
    let samples = Error::Config {
        path,
        source: crate::config::Error::ManySamples {
            step: 1,
            frames: 31,
            samples: 31,
        },
    };
    let reasons = [size, range, reduction, samples].map(|error| waits_for_the_generic_engine(&error).unwrap());
    assert!(reasons[0].contains("720×540"));
    assert!(reasons[1].contains("full range"));
    assert!(reasons[2].contains("3:2"));
    assert!(reasons[3].contains("more than 16"));
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
    let small = Error::Geometry {
        path: path.clone(),
        source: crate::geometry::Error::TooSmall { width: 8, height: 6 },
    };
    let silent = Error::Input(input::Error::NoAnnouncedFrames { path });
    for error in [few, small, silent] {
        assert_eq!(waits_for_the_generic_engine(&error), None, "{error}");
    }
}
