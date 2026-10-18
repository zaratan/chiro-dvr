#[path = "shared/comparison.rs"]
mod comparison;

use comparison::Tracks;
use reference::location::{Reference, recording_directories};
use reference::recording::Recording;
use types::rate::FrameRate;

const fn changed(
    reference: usize,
    tracked: usize,
    paired: usize,
    agreeing: usize,
    lost: usize,
    added: usize,
) -> Tracks {
    Tracks {
        reference,
        tracked,
        paired,
        agreeing,
        lost,
        added,
    }
}

const FROZEN: [(&str, Tracks); 26] = [
    ("video_089.mp4", Tracks::same(1)),
    ("video_090.mp4", changed(20, 18, 17, 16, 3, 1)),
    ("video_091.mp4", Tracks::same(21)),
    ("video_092.mp4", Tracks::same(14)),
    ("video_093.mp4", changed(6, 4, 3, 1, 3, 1)),
    ("video_094.mp4", changed(14, 12, 11, 10, 3, 1)),
    ("video_095.mp4", changed(78, 67, 59, 50, 19, 8)),
    ("video_096.mp4", changed(136, 121, 117, 100, 19, 4)),
    ("video_097.mp4", changed(149, 136, 124, 112, 25, 12)),
    ("video_098.mp4", changed(131, 126, 116, 105, 15, 10)),
    ("video_099.mp4", changed(117, 109, 104, 98, 13, 5)),
    ("video_100.mp4", changed(98, 92, 87, 82, 11, 5)),
    ("video_101.mp4", changed(32, 31, 29, 26, 3, 2)),
    ("video_102.mp4", changed(36, 35, 34, 31, 2, 1)),
    ("video_103.mp4", changed(10, 10, 10, 9, 0, 0)),
    ("video_120.mp4", Tracks::same(0)),
    ("video_121.mp4", Tracks::same(2)),
    ("video_122.mp4", Tracks::same(0)),
    ("video_123.mp4", Tracks::same(8)),
    ("video_124.mp4", Tracks::same(5)),
    ("video_125.mp4", changed(47, 44, 43, 40, 4, 1)),
    ("video_126.mp4", Tracks::same(13)),
    ("video_127.mp4", changed(7, 7, 6, 6, 1, 1)),
    ("video_128.mp4", Tracks::same(5)),
    ("video_129.mp4", changed(16, 15, 15, 14, 1, 0)),
    ("video_130.mp4", changed(6, 6, 5, 5, 1, 1)),
];

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_track_their_v020_detections_into_their_frozen_tracks() {
    let mut identical = 0;
    let found: Vec<(String, Tracks)> = recording_directories(Reference::V020)
        .unwrap()
        .iter()
        .map(|directory| {
            let recording = Recording::read(directory).unwrap();
            let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
            let found = comparison::compare(&recording, rate).unwrap();
            println!(
                "{}: {:?}, {} identical, {} with other frames, largest gap {:e}",
                found.video,
                found.tracks(),
                found.identical,
                found.different_frames,
                found.largest_gap
            );
            for discrepancy in &found.discrepancies {
                println!("  {discrepancy:?}");
            }
            identical += found.identical;
            (found.video.clone(), found.tracks())
        })
        .collect();
    println!("identical in all: {identical}");

    let frozen: Vec<(String, Tracks)> = FROZEN
        .iter()
        .map(|(name, tracks)| ((*name).to_owned(), *tracks))
        .collect();
    assert_eq!(found, frozen);
}
