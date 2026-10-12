use std::io;

use compare::case::{Case, WHOLE};
use compare::comparison::compare;
use compare::report::write_report;

#[test]
#[ignore = "reference"]
fn the_whole_092_agrees_with_the_reference_on_98_percent_of_its_detections_both_ways() {
    let comparison = compare(&Case::committed(WHOLE), None).unwrap();

    write_report(std::slice::from_ref(&comparison), &mut io::stdout().lock()).unwrap();
    assert!(comparison.detections.holds());
}
