use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use types::detection::Detection;
use types::frame::FrameIndex;
use types::language::Language;
use types::track::Track;

use super::{Names, params_json};

fn track(id: usize) -> Track {
    let point = Detection {
        frame: FrameIndex::new(0),
        x: 1.0,
        y: 1.0,
        left: 0.0,
        top: 0.0,
        width: 3.0,
        height: 3.0,
        area: 9.0,
        amplitude: 80.0,
    };
    Track::new(id, vec![point]).unwrap()
}

fn every_name(language: Language) -> Vec<PathBuf> {
    let (names, dest, stem) = (Names::new(language), Path::new("out/v"), OsStr::new("v"));
    let clips = names.clips_dir(dest);
    vec![
        clips.join(names.clip(&track(1), 0.0, 1)),
        clips.join(names.zoomed_clip(&track(1), 0.0, 1)),
        names.tracks_csv(dest, stem),
        names.summary_png(dest, stem, None),
        names.annotated_video(dest, stem),
    ]
}

#[test]
fn english_names_for_an_english_reader() {
    assert_eq!(
        every_name(Language::English),
        [
            "out/v/clips/track_01_0m00s.mp4",
            "out/v/clips/track_01_0m00s_zoom.mp4",
            "out/v/v_tracks.csv",
            "out/v/v_summary.png",
            "out/v/v_annotated.mp4",
        ]
        .map(PathBuf::from)
    );
}

#[test]
fn french_names_without_any_accent_that_apfs_would_decompose() {
    let names = every_name(Language::French);
    assert_eq!(
        names,
        [
            "out/v/extraits/piste_01_0m00s.mp4",
            "out/v/extraits/piste_01_0m00s_zoom.mp4",
            "out/v/v_pistes.csv",
            "out/v/v_resume.png",
            "out/v/v_annotee.mp4",
        ]
        .map(PathBuf::from)
    );
    assert!(names.iter().all(|name| name.to_string_lossy().is_ascii()));
}

#[test]
fn params_json_keeps_its_english_name_in_french() {
    assert_eq!(params_json(Path::new("out/v")), PathBuf::from("out/v/params.json"));
}

#[test]
fn clip_time_drops_hundredths_and_the_track_number_tells_apart_two_tracks_of_the_same_second() {
    let names = Names::new(Language::English);
    assert_eq!(
        [names.clip(&track(7), 237.4, 8), names.clip(&track(8), 237.97, 8)],
        ["track_07_3m57s.mp4", "track_08_3m57s.mp4"]
    );
}

#[test]
fn clip_time_counts_minutes_past_the_hour() {
    assert_eq!(
        Names::new(Language::English).clip(&track(12), 3725.0, 12),
        "track_12_62m05s.mp4"
    );
}

#[test]
fn track_number_takes_the_width_of_the_track_count_so_the_finder_sorts_clips_in_numeric_order() {
    let names = Names::new(Language::English);
    for (track_count, expected) in [
        (99, "track_07_0m00s.mp4"),
        (100, "track_007_0m00s.mp4"),
        (1000, "track_0007_0m00s.mp4"),
    ] {
        assert_eq!(names.clip(&track(7), 0.0, track_count), expected);
    }
}

#[test]
fn french_and_zoomed_clips_follow_the_same_number_width() {
    let names = Names::new(Language::French);
    assert_eq!(
        [names.clip(&track(7), 0.0, 100), names.zoomed_clip(&track(7), 0.0, 100)],
        ["piste_007_0m00s.mp4", "piste_007_0m00s_zoom.mp4"]
    );
}

#[test]
fn summary_of_a_long_video_carries_its_period_in_minutes() {
    let summary = Names::new(Language::English).summary_png(Path::new("out/v"), OsStr::new("v"), Some(0..10));
    assert_eq!(summary, PathBuf::from("out/v/v_summary_000m-010m.png"));
}

#[test]
fn a_time_before_the_start_of_the_video_is_named_as_zero() {
    assert_eq!(
        Names::new(Language::English).clip(&track(1), -0.4, 1),
        "track_01_0m00s.mp4"
    );
}

#[test]
fn an_accented_video_name_is_kept_as_given() {
    let csv = Names::new(Language::French).tracks_csv(Path::new("out"), OsStr::new("nuit_été"));
    assert_eq!(csv, PathBuf::from("out/nuit_été_pistes.csv"));
}
