use reference::location::{Reference, committed};
use reference::recording::Recording;
use tracking::exclusion::{Exclusion, exclude};
use tracking::stability::StabilityConfig;
use types::rate::FrameRate;

fn excluded(recording: &Recording, rate: FrameRate) -> Exclusion {
    let summary = &recording.summary;
    exclude(
        recording.frames.detections(),
        summary.damaged_spans.clone(),
        summary.counts.damaged_frames_reported,
        rate,
        StabilityConfig::default(),
    )
}

#[test]
fn the_excerpt_and_the_full_092_keep_every_detection_as_the_reference_does() {
    for (name, kept) in [("video_092_original_3m24-4m05", 402), ("video_092_original", 780)] {
        let recording = Recording::read(&committed(Reference::V011, name)).unwrap();
        let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
        let exclusion = excluded(&recording, rate);
        assert_eq!(exclusion.unstable, recording.summary.unstable_spans);
        assert_eq!(exclusion.unstable, []);
        assert_eq!(
            exclusion.detections.detection_count(),
            recording.summary.counts.detections_kept
        );
        assert_eq!(exclusion.detections.detection_count(), kept);
        assert_eq!(exclusion.ignored_frame_count, 0);
    }
}
