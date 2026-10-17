use std::io;

use compare::agreement::Agreement;
use compare::case::Target;
use compare::comparison::{Comparison, Folders, VIDEOS_AT_ONCE, compare_all};
use compare::report::write_report;
use compare::tracks::TrackCounts;
use reference::location::Reference;

const V011_RATE: f64 = 0.98;

fn within_the_v011_rate(detections: &Agreement) -> bool {
    detections.reference_rate() >= V011_RATE && detections.candidate_rate() >= V011_RATE
}

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_keep_98_percent_of_their_detections_and_lose_three_of_the_966_tracks() {
    let cases = Target::Recorded.cases(Reference::V011).unwrap();
    let comparisons: Vec<Comparison> = compare_all(&cases, VIDEOS_AT_ONCE, &Folders::default())
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    write_report(&comparisons, &mut io::stdout().lock()).unwrap();
    let below: Vec<&str> = comparisons
        .iter()
        .filter(|comparison| !within_the_v011_rate(&comparison.detections))
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
