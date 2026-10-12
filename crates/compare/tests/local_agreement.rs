use std::io;

use compare::case::Target;
use compare::comparison::{Comparison, VIDEOS_AT_ONCE, compare_all};
use compare::report::write_report;
use compare::tracks::TrackCounts;

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_keep_98_percent_of_their_detections_and_lose_three_of_the_966_tracks() {
    let cases = Target::Recorded.cases().unwrap();
    let comparisons: Vec<Comparison> = compare_all(&cases, VIDEOS_AT_ONCE, None)
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    write_report(&comparisons, &mut io::stdout().lock()).unwrap();
    let below: Vec<&str> = comparisons
        .iter()
        .filter(|comparison| !comparison.detections.holds())
        .map(|comparison| comparison.detections.name())
        .collect();
    let TrackCounts {
        reference,
        paired,
        agreeing,
        lost,
        added,
    } = comparisons.iter().map(|comparison| comparison.tracks.counts()).sum();
    assert_eq!(comparisons.len(), 26);
    assert_eq!(below, Vec::<&str>::new());
    assert_eq!([reference, paired, agreeing, lost, added], [966, 963, 958, 3, 9]);
}
