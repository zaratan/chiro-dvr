use types::frame::FrameIndex;
use types::track::Track;

use super::Recording;
use crate::location::committed;

const EXCERPT: &str = "video_092_original_3m24-4m05";
const FULL: &str = "video_092_original";

fn recording(name: &str) -> Recording {
    Recording::read(&committed(name)).unwrap()
}

fn starts(recording: &Recording) -> Vec<usize> {
    recording.tracks.iter().map(|track| track.first().frame.get()).collect()
}

fn point_counts(recording: &Recording) -> Vec<usize> {
    recording.tracks.iter().map(|track| track.points().len()).collect()
}

fn points_outside_raw_detections(recording: &Recording) -> usize {
    recording
        .tracks
        .iter()
        .flat_map(Track::points)
        .filter(|point| {
            let raw = recording.frames.detections().get(point.frame).unwrap_or_default();
            !raw.contains(point)
        })
        .count()
}

#[test]
fn the_excerpt_reads_with_its_raw_detections_and_tracks() {
    let excerpt = recording(EXCERPT);
    assert_eq!(excerpt.summary.video.name, "video_092_original_3m24-4m05.mp4");
    assert_eq!(excerpt.frames.entries().len(), 1230);
    assert_eq!(excerpt.frames.detections().detection_count(), 402);
    assert_eq!(starts(&excerpt), [144, 705, 783, 972]);
    assert_eq!(point_counts(&excerpt), [290, 17, 24, 27]);
    assert_eq!(excerpt.rows.len(), 4);
}

#[test]
fn the_full_092_reads_with_its_fourteen_tracks() {
    let full = recording(FULL);
    assert_eq!(full.frames.entries().len(), 9011);
    assert_eq!(full.frames.detections().frame_count(), 9011);
    assert_eq!(full.frames.detections().detection_count(), 780);
    assert_eq!(
        starts(&full),
        [
            222, 1019, 1550, 1614, 2180, 3271, 3688, 4343, 4714, 4923, 6294, 6855, 6933, 7122
        ]
    );
    assert_eq!(
        point_counts(&full),
        [20, 40, 7, 34, 35, 32, 17, 13, 10, 44, 290, 17, 24, 27]
    );
    assert_eq!(
        point_counts(&full).iter().sum::<usize>(),
        full.summary.counts.track_points
    );
}

#[test]
fn merged_twin_points_are_not_raw_detections() {
    assert_eq!(points_outside_raw_detections(&recording(EXCERPT)), 17);
    assert_eq!(points_outside_raw_detections(&recording(FULL)), 63);
}

#[test]
fn both_recordings_give_back_the_same_values_once_rewritten() {
    for name in [EXCERPT, FULL] {
        let directory = tempfile::tempdir().unwrap();
        let read = recording(name);
        read.write(directory.path()).unwrap();
        assert_eq!(Recording::read(directory.path()).unwrap(), read);
    }
}

#[test]
fn a_recording_rewritten_with_compressed_frames_reads_the_same() {
    let directory = tempfile::tempdir().unwrap();
    let mut excerpt = recording(EXCERPT);
    excerpt.summary.files.frames = "frames.jsonl.gz".to_owned();
    excerpt.write(directory.path()).unwrap();
    assert!(directory.path().join("frames.jsonl.gz").is_file());
    assert_eq!(Recording::read(directory.path()).unwrap(), excerpt);
}

#[test]
fn track_measures_agree_with_the_table_written_by_the_reference() {
    let full = recording(FULL);
    for (track, row) in full.tracks.iter().zip(&full.rows) {
        assert_eq!((track.id(), track.points().len()), (row.id, row.hits));
        assert!(
            (track.chord() - row.chord_px).abs() <= 0.05,
            "chord of track {}",
            row.id
        );
        assert!(
            (track.path_length() - row.path_px).abs() <= 0.05,
            "path of track {}",
            row.id
        );
        assert!(
            (track.max_area() - row.max_area_px as f64).abs() <= 0.5,
            "area of track {}",
            row.id
        );
        assert!(
            (track.peak_amplitude() - row.peak_amplitude).abs() <= 0.05,
            "amplitude of track {}",
            row.id
        );
    }
}

#[test]
fn rate_times_give_back_the_start_times_of_the_table() {
    let full = recording(FULL);
    let timeline = full.timeline().unwrap();
    assert_eq!(timeline.frame_count(), 9011);
    assert_eq!(timeline.pts(FrameIndex::new(1)).map(types::frame::Pts::get), Some(333));
    for (track, row) in full.tracks.iter().zip(&full.rows) {
        assert!((timeline.seconds_by_rate(track.first().frame) - row.start_s).abs() <= 0.005);
    }
}

#[test]
fn a_recording_with_an_unknown_frame_rate_has_no_timeline() {
    let mut excerpt = recording(EXCERPT);
    excerpt.summary.stream.avg_frame_rate = types::rational::Rational::new(0, 1).unwrap();
    assert!(matches!(excerpt.timeline(), Err(crate::error::Error::Rate(_))));
    excerpt.summary.stream.avg_frame_rate = types::rational::Rational::new(30, 1).unwrap();
    excerpt.summary.stream.time_base = types::rational::Rational::new(0, 1).unwrap();
    assert!(matches!(excerpt.timeline(), Err(crate::error::Error::Timeline(_))));
}
