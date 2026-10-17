use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use engine::config::DetectConfig;
use engine::detection::{Detected, detect};
use engine::input::Input;
use engine::settings::DetectSettings;
use media::decode::{ActiveThreads, Threads};
use reference::fixture::fixture;
use reference::video::{DECODED_FORMAT, DecoderThreads, VideoSummary};
use tempfile::TempDir;
use tracking::settings::{StabilitySettings, TrackSettings};
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

use super::{followed, same_stream, summary, threads};
use crate::testing::{SPOT, WORK_WIDTH, build, probed};

fn config() -> DetectConfig {
    DetectConfig::try_from(&DetectSettings {
        work_width: WORK_WIDTH,
        ..DetectSettings::default()
    })
    .unwrap()
}

fn wiped_spot(folder: &Path) -> PathBuf {
    let path = folder.join(SPOT);
    fs::copy(fixture(SPOT).unwrap(), &path).unwrap();
    media::payload::wipe(&path, 10..12, 20).unwrap();
    path
}

fn described(path: &Path, threads: Threads) -> (Detected, VideoSummary) {
    let detected = detect(path, threads, &config()).unwrap();
    let followed = followed(&detected).unwrap();
    let summary = summary(probed(path), &detected, &followed, &config(), &build()).unwrap();
    (detected, summary)
}

#[test]
fn a_damaged_video_is_described_by_the_frames_its_own_decoder_flags_and_the_spans_it_ignores() {
    let folder = TempDir::new().unwrap();
    let (detected, summary) = described(&wiped_spot(folder.path()), Threads::Exactly(NonZeroUsize::MIN));
    let damage = detected.damage();
    let decoder = summary.decoder.as_ref().unwrap();

    assert_ne!(damage.spans(), []);
    assert_eq!(
        (decoder.error_frames.as_slice(), decoder.gap_frames.as_slice()),
        (damage.error_frames(), damage.jump_frames())
    );
    assert_eq!(summary.damaged_spans, damage.spans());
    assert_eq!(summary.counts.damaged_frames_reported, damage.reported_frames());
    assert_eq!(decoder.threads, DecoderThreads::Single);
    assert_eq!(
        (summary.format_version, summary.reference.as_str()),
        (DECODED_FORMAT, "v0.2.0")
    );
}

#[test]
fn the_counts_of_a_recording_are_those_of_its_detections_and_tracks() {
    let (detected, summary) = described(&fixture(SPOT).unwrap(), Threads::Automatic);
    let followed = followed(&detected).unwrap();

    assert_eq!(summary.counts.frames, 40);
    assert_eq!(summary.counts.detections, detected.detections().detection_count());
    assert_eq!(
        summary.counts.detections_kept,
        followed.exclusion.detections.detection_count()
    );
    assert_eq!(summary.counts.tracks, 1);
    assert_eq!(summary.counts.track_points, followed.tracks[0].points().len());
    assert_eq!(summary.unstable_spans, followed.exclusion.unstable);
    assert_eq!(summary.background_half_window, detected.half_window().get());
}

#[test]
fn the_settings_and_versions_recorded_are_those_the_engine_ran_with_and_the_binary_was_built_from() {
    let (_, summary) = described(&fixture(SPOT).unwrap(), Threads::Automatic);
    let config = &summary.recorded["config"];
    let versions = &summary.recorded["versions"];

    assert_eq!(
        summary.detect_settings::<DetectSettings>().unwrap(),
        DetectSettings::from(&self::config())
    );
    assert_eq!(
        serde_json::from_value::<TrackSettings>(config["TrackConfig"].clone()).unwrap(),
        TrackSettings::from(TrackConfig::default())
    );
    assert_eq!(
        serde_json::from_value::<StabilitySettings>(config["StabilityConfig"].clone()).unwrap(),
        StabilitySettings::from(StabilityConfig::default())
    );
    assert_eq!(
        (&versions["batdetect"], &versions["commit"], &versions["rustc"]),
        (
            &serde_json::json!("0.2.0-1-g28d5526"),
            &serde_json::json!("28d5526d0b2c"),
            &serde_json::json!("rustc 1.90.0")
        )
    );
    assert_eq!(versions["ffmpeg"], media::library::version().as_ref());
    assert_eq!(summary.recorded.keys().collect::<Vec<_>>(), ["config", "versions"]);
}

#[test]
fn a_stream_other_than_the_one_the_v011_recorded_is_refused() {
    let path = fixture(SPOT).unwrap();
    let video = Input::open(&path, Threads::Automatic).unwrap().video().clone();
    let mut wider = probed(&path);
    wider.stream.width += 1;
    let mut faster = probed(&path);
    faster.stream.avg_frame_rate = types::rational::Rational::new(60, 1).unwrap();

    assert!(same_stream(&probed(&path), &video).is_ok());
    let refusals: Vec<String> = [wider, faster]
        .iter()
        .map(|refused| same_stream(refused, &video).unwrap_err().to_string())
        .collect();
    assert_eq!(
        refusals,
        [
            format!(
                "{}: media reads a stream of 320×240 at 30/1 frames per second, time base 1/15360, the v0.1.1 recorded 321×240 at 30/1 frames per second, time base 1/15360",
                path.display()
            ),
            format!(
                "{}: media reads a stream of 320×240 at 30/1 frames per second, time base 1/15360, the v0.1.1 recorded 320×240 at 60/1 frames per second, time base 1/15360",
                path.display()
            ),
        ]
    );
}

#[test]
fn each_kind_of_decoding_threads_is_recorded_as_the_same_kind_with_its_count() {
    let four = NonZeroUsize::new(4).unwrap();

    assert_eq!(
        [
            ActiveThreads::Single,
            ActiveThreads::Frame(four),
            ActiveThreads::Slice(four)
        ]
        .map(threads),
        [
            DecoderThreads::Single,
            DecoderThreads::Frame(four),
            DecoderThreads::Slice(four)
        ]
    );
}
