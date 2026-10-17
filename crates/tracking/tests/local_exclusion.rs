#[path = "shared/kept.rs"]
mod kept;

use kept::{Kept, UNCHANGED};
use reference::location::{Reference, recording_directories};
use reference::recording::Recording;
use types::rate::FrameRate;

const FROZEN: [(&str, Kept); 26] = [
    ("video_089.mp4", UNCHANGED),
    ("video_090.mp4", UNCHANGED),
    ("video_091.mp4", UNCHANGED),
    ("video_092.mp4", UNCHANGED),
    ("video_093.mp4", UNCHANGED),
    ("video_094.mp4", UNCHANGED),
    ("video_095.mp4", UNCHANGED),
    ("video_096.mp4", UNCHANGED),
    ("video_097.mp4", UNCHANGED),
    ("video_098.mp4", UNCHANGED),
    ("video_099.mp4", UNCHANGED),
    ("video_100.mp4", UNCHANGED),
    ("video_101.mp4", UNCHANGED),
    ("video_102.mp4", UNCHANGED),
    ("video_103.mp4", UNCHANGED),
    ("video_120.mp4", UNCHANGED),
    ("video_121.mp4", UNCHANGED),
    ("video_122.mp4", UNCHANGED),
    ("video_123.mp4", UNCHANGED),
    ("video_124.mp4", UNCHANGED),
    ("video_125.mp4", UNCHANGED),
    ("video_126.mp4", UNCHANGED),
    ("video_127.mp4", UNCHANGED),
    ("video_128.mp4", UNCHANGED),
    ("video_129.mp4", UNCHANGED),
    ("video_130.mp4", UNCHANGED),
];

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_keep_the_detections_and_unstable_spans_of_the_v020_up_to_their_frozen_gaps() {
    let found: Vec<(String, Kept)> = recording_directories(Reference::V020)
        .unwrap()
        .iter()
        .map(|directory| {
            let recording = Recording::read(directory).unwrap();
            let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
            (recording.summary.video.name.clone(), kept::kept(&recording, rate))
        })
        .collect();

    let frozen: Vec<(String, Kept)> = FROZEN.iter().map(|(name, kept)| ((*name).to_owned(), *kept)).collect();
    assert_eq!(found, frozen);
}
