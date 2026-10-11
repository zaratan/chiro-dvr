#[path = "shared/comparison.rs"]
mod comparison;

use reference::location::committed;
use reference::recording::Recording;
use types::rate::FrameRate;

#[test]
fn the_excerpt_and_the_full_092_give_exactly_the_tracks_of_the_reference() {
    for (name, expected) in [("video_092_original_3m24-4m05", 4), ("video_092_original", 14)] {
        let recording = Recording::read(&committed(name)).unwrap();
        let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
        let found = comparison::compare(&recording, rate).unwrap();
        println!(
            "{}: {} pairs, {} identical, {} with other frames, largest gap {:e}",
            found.video, found.pairs, found.identical, found.different_frames, found.largest_gap
        );
        assert_eq!((found.reference, found.tracked.len()), (expected, expected));
        assert_eq!((found.pairs, found.agreeing), (expected, expected));
        assert_eq!((found.reference_only, found.tracked_only), (0, 0));
        assert_eq!(found.discrepancies, []);
        assert_eq!(found.identical, expected);
        let numbers: Vec<usize> = found.tracked.iter().map(types::track::Track::id).collect();
        assert_eq!(numbers, (1..=expected).collect::<Vec<_>>());
    }
}
