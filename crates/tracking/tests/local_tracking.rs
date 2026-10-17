#[path = "shared/comparison.rs"]
mod comparison;

use comparison::Tracks;
use reference::location::{Reference, recording_directories};
use reference::recording::Recording;
use types::rate::FrameRate;

const fn rejoined(reference: usize, joins: usize, tracks_joined: usize) -> Tracks {
    let tracked = reference - joins;
    Tracks {
        reference,
        tracked,
        paired: tracked,
        agreeing: tracked - tracks_joined,
        lost: joins,
        added: 0,
    }
}

const FROZEN: [(&str, Tracks); 26] = [
    ("video_089.mp4", Tracks::same(1)),
    ("video_090.mp4", Tracks::same(20)),
    ("video_091.mp4", Tracks::same(21)),
    ("video_092.mp4", Tracks::same(14)),
    ("video_093.mp4", rejoined(6, 2, 2)),
    ("video_094.mp4", rejoined(14, 1, 1)),
    ("video_095.mp4", rejoined(78, 11, 11)),
    ("video_096.mp4", rejoined(136, 13, 12)),
    ("video_097.mp4", rejoined(149, 15, 11)),
    ("video_098.mp4", rejoined(131, 8, 7)),
    ("video_099.mp4", rejoined(117, 7, 7)),
    ("video_100.mp4", rejoined(98, 5, 5)),
    ("video_101.mp4", rejoined(32, 1, 1)),
    ("video_102.mp4", rejoined(36, 1, 1)),
    ("video_103.mp4", Tracks::same(10)),
    ("video_120.mp4", Tracks::same(0)),
    ("video_121.mp4", Tracks::same(2)),
    ("video_122.mp4", Tracks::same(0)),
    ("video_123.mp4", Tracks::same(8)),
    ("video_124.mp4", Tracks::same(5)),
    ("video_125.mp4", rejoined(47, 3, 3)),
    ("video_126.mp4", Tracks::same(13)),
    ("video_127.mp4", Tracks::same(7)),
    ("video_128.mp4", Tracks::same(5)),
    ("video_129.mp4", Tracks::same(16)),
    ("video_130.mp4", Tracks::same(6)),
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
