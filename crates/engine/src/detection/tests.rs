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
