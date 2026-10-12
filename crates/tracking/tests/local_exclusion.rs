use reference::location::recording_directories;
use reference::recording::Recording;
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use types::rate::FrameRate;

#[test]
#[ignore = "local"]
fn the_twenty_six_recorded_videos_keep_the_detections_and_spans_of_the_reference() {
    let directories = recording_directories().unwrap();
    assert_eq!(directories.len(), 26);
    let mut differences = Vec::new();
    for directory in directories {
        let recording = Recording::read(&directory).unwrap();
        let summary = &recording.summary;
        let exclusion = exclude(
            recording.frames.detections(),
            summary.damaged_spans.clone(),
            summary.counts.damaged_frames_reported,
            FrameRate::new(summary.stream.avg_frame_rate).unwrap(),
            StabilityConfig::default(),
        );
        let kept = exclusion.detections.detection_count();
        if exclusion.unstable != summary.unstable_spans || kept != summary.counts.detections_kept {
            differences.push(format!(
                "{}: kept {kept} instead of {}, unstable {:?} instead of {:?}",
                summary.video.name, summary.counts.detections_kept, exclusion.unstable, summary.unstable_spans
            ));
        }
    }
    assert_eq!(differences, Vec::<String>::new());
}
