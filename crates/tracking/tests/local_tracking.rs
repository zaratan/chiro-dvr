#[path = "shared/comparison.rs"]
mod comparison;

use reference::location::recording_directories;
use reference::recording::Recording;
use types::rate::FrameRate;

#[test]
#[ignore = "local"]
fn the_twenty_six_recorded_videos_give_the_tracks_of_the_reference() {
    let directories = recording_directories().unwrap();
    assert_eq!(directories.len(), 26);
    let (mut reference, mut tracked, mut pairs, mut identical) = (0, 0, 0, 0);
    let mut discrepancies = Vec::new();
    for directory in directories {
        let recording = Recording::read(&directory).unwrap();
        let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
        let found = comparison::compare(&recording, rate).unwrap();
        println!(
            "{}: reference {}, tracked {}, pairs {} (agreeing {}), alone {} / {}, identical {}/{}, other frames {}, largest gap {:e}",
            found.video,
            found.reference,
            found.tracked.len(),
            found.pairs,
            found.agreeing,
            found.reference_only,
            found.tracked_only,
            found.identical,
            found.pairs,
            found.different_frames,
            found.largest_gap
        );
        reference += found.reference;
        tracked += found.tracked.len();
        pairs += found.pairs;
        identical += found.identical;
        discrepancies.extend(found.discrepancies);
    }
    println!("total: reference {reference}, tracked {tracked}, pairs {pairs}, identical {identical}");
    for discrepancy in &discrepancies {
        println!("{discrepancy:?}");
    }
    assert_eq!(reference, 966);
    assert_eq!(discrepancies, []);
}
