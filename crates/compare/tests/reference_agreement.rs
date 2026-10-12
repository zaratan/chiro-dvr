use std::io;

use compare::case::{Case, WHOLE, WHOLE_AT_1440};
use compare::comparison::compare;
use compare::report::write_report;
use compare::tracks::TrackCounts;
use reference::location::committed;

#[test]
#[ignore = "reference"]
fn the_whole_092_agrees_with_the_reference_on_98_percent_of_its_detections_both_ways() {
    let comparison = compare(&Case::committed(WHOLE), None).unwrap();

    write_report(std::slice::from_ref(&comparison), &mut io::stdout().lock()).unwrap();
    assert!(comparison.detections.holds());
}

#[test]
#[ignore = "reference"]
fn the_whole_092_worked_at_1440_pixels_agrees_with_its_reference_on_98_percent_of_its_detections_and_on_every_track() {
    let comparison = compare(&Case::setting(committed(WHOLE_AT_1440)), None).unwrap();

    write_report(std::slice::from_ref(&comparison), &mut io::stdout().lock()).unwrap();
    assert!(comparison.detections.holds());
    assert_eq!(
        comparison.tracks.counts(),
        TrackCounts {
            reference: 14,
            paired: 14,
            agreeing: 14,
            lost: 0,
            added: 0,
        }
    );
}
