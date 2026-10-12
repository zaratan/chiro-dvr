use std::num::NonZeroUsize;

use media::decode::{ActiveThreads, Threads};
use tempfile::TempDir;
use types::rational::Rational;

use super::{Error, detect};
use crate::config::DetectConfig;
use crate::settings::{DetectSettings, Region};
use crate::synthetic::{luma, with_square};
use crate::testing::{self, FRAMES, binoculars};

const SOURCE: (u32, u32) = (360, 270);

fn spot(index: usize) -> Vec<u8> {
    with_square(vec![luma(200.0); 360 * 270], 360, (60 + 6 * index, 135), 9, -80.0)
}

fn three_to_two() -> DetectConfig {
    masked_at_three_to_two(Vec::new())
}

fn masked_at_three_to_two(osd_regions: Vec<Region>) -> DetectConfig {
    DetectConfig::try_from(&DetectSettings {
        work_width: 240,
        osd_regions,
        ..DetectSettings::default()
    })
    .unwrap()
}

fn top_band(height: f64) -> Region {
    Region {
        x0: 0.0,
        y0: 0.0,
        x1: 1.0,
        y1: height,
    }
}

fn found_per_frame(path: &std::path::Path, config: &DetectConfig) -> Vec<usize> {
    let detected = detect(path, Threads::Automatic, config).unwrap();
    detected.detections().iter().map(|(_, found)| found.len()).collect()
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

#[test]
fn a_spot_inside_the_display_band_is_never_seen() {
    let folder = TempDir::new().unwrap();
    let path = binoculars(
        &folder.path().join("band.mp4"),
        SOURCE,
        Rational::new(30, 1).unwrap(),
        (0..20).map(|index| with_square(vec![luma(200.0); 360 * 270], 360, (60 + 6 * index, 4), 9, -80.0)),
    );

    assert_eq!(found_per_frame(&path, &three_to_two()), [1; 20]);
    assert_eq!(
        found_per_frame(&path, &masked_at_three_to_two(vec![top_band(0.07)])),
        [0; 20]
    );
}

#[test]
fn a_flickering_display_band_under_the_mask_leaves_the_gain_unchanged() {
    let folder = TempDir::new().unwrap();
    let flickering = (0..20).map(|index| {
        let mut plane = spot(index);
        plane[..360 * 68].fill(if index % 2 == 0 { 16 } else { 235 });
        plane
    });
    let path = binoculars(
        &folder.path().join("flicker.mp4"),
        SOURCE,
        Rational::new(30, 1).unwrap(),
        flickering,
    );

    assert_eq!(
        found_per_frame(&path, &masked_at_three_to_two(vec![top_band(0.25)])),
        [1; 20]
    );
}

#[test]
fn display_regions_covering_the_whole_frame_are_refused_before_any_detection() {
    let folder = TempDir::new().unwrap();
    let path = binoculars(
        &folder.path().join("covered.mp4"),
        SOURCE,
        Rational::new(30, 1).unwrap(),
        (0..3).map(spot),
    );
    let halves = vec![
        top_band(0.5),
        Region {
            x0: 0.0,
            y0: 0.5,
            x1: 1.0,
            y1: 1.0,
        },
    ];

    let refused = detect(&path, Threads::Automatic, &masked_at_three_to_two(halves)).unwrap_err();

    assert!(matches!(refused, Error::Mask { .. }), "{refused:?}");
    assert_eq!(
        refused.to_string(),
        format!("cannot mask the OSD regions of {}", path.display())
    );
}
