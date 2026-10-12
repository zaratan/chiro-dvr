use std::fs;

use types::frame::FrameIndex;
use types::rational::Rational;

use super::VideoSummary;
use crate::error::Error;
use crate::location::committed;

fn summary_092() -> VideoSummary {
    VideoSummary::read(&committed("video_092_original").join("video.json")).unwrap()
}

#[test]
fn the_stream_of_the_092_reads_as_exact_rationals() {
    let stream = summary_092().stream;
    assert_eq!((stream.width, stream.height), (1440, 1080));
    assert_eq!(stream.avg_frame_rate, Rational::new(90_110_000, 3_000_927).unwrap());
    assert_eq!(stream.r_frame_rate, Rational::new(10_000, 333).unwrap());
    assert_eq!(stream.time_base, Rational::new(1, 10_000).unwrap());
    assert_eq!(stream.nb_frames.as_deref(), Some("9011"));
}

#[test]
fn the_092_summary_reports_no_damage_and_its_counts() {
    let summary = summary_092();
    assert_eq!(summary.ffprobe.frame_count, 9011);
    assert!(summary.ffprobe.error_frames.is_empty() && summary.damaged_spans.is_empty());
    assert_eq!(summary.background_half_window, 15);
    assert_eq!((summary.counts.detections, summary.counts.tracks), (780, 14));
}

#[test]
fn sections_without_a_type_are_kept_as_written() {
    let recorded = summary_092().recorded;
    assert_eq!(recorded.len(), 3);
    assert_eq!(recorded["config"]["TrackConfig"]["max_gap"], 6);
    assert_eq!(recorded["versions"]["batdetect"], "0.1.1");
}

#[test]
fn a_summary_survives_a_round_trip() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("video.json");
    let mut summary = summary_092();
    summary.damaged_spans = vec![types::span::Span::new(FrameIndex::new(1286), FrameIndex::new(1328)).unwrap()];
    summary.stream.nb_frames = None;
    summary.write(&path).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.ends_with("}\n") && !text.contains("nb_frames"));
    assert_eq!(VideoSummary::read(&path).unwrap(), summary);
}

#[test]
fn another_format_version_is_refused_before_its_fields_are_read() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("video.json");
    fs::write(&path, "{\"format_version\": 2, \"frames\": []}\n").unwrap();
    let error = VideoSummary::read(&path).unwrap_err();
    assert!(matches!(
        error,
        Error::Version {
            found: 2,
            supported: 1,
            ..
        }
    ));
}

#[test]
fn a_malformed_summary_is_reported_with_its_line() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("video.json");
    fs::write(&path, "{\n  \"format_version\": \"one\"\n}\n").unwrap();
    assert!(matches!(
        VideoSummary::read(&path).unwrap_err(),
        Error::Json { line: 2, .. }
    ));
}

#[test]
fn a_missing_summary_or_folder_is_reported() {
    let directory = tempfile::tempdir().unwrap();
    let absent = directory.path().join("absent").join("video.json");
    assert!(matches!(VideoSummary::read(&absent).unwrap_err(), Error::Read { .. }));
    assert!(matches!(summary_092().write(&absent).unwrap_err(), Error::Write { .. }));
}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct Areas {
    min_area: f64,
    max_area: f64,
}

#[test]
fn the_detection_settings_of_a_recording_read_into_the_type_the_caller_asks_for() {
    let areas: Areas = summary_092().detect_settings().unwrap();
    assert_eq!(
        areas,
        Areas {
            min_area: 4.0,
            max_area: 2700.0
        }
    );
}

#[test]
fn a_recording_without_detection_settings_says_so() {
    let mut summary = summary_092();
    summary.recorded.remove("config");
    assert!(matches!(summary.detect_settings::<Areas>(), Err(Error::NoDetectConfig)));
}

#[test]
fn detection_settings_of_another_shape_are_refused_with_their_cause() {
    let error = summary_092().detect_settings::<Vec<f64>>().unwrap_err();
    assert!(matches!(error, Error::DetectConfig(_)));
    assert!(std::error::Error::source(&error).is_some());
}
