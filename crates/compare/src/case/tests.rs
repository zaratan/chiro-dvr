use std::path::PathBuf;

use super::{Case, EXTRACT, Target, WHOLE};

#[test]
fn the_extract_and_the_whole_092_are_read_from_the_committed_reference() {
    let [extract] = Target::Extract.cases().unwrap().try_into().unwrap();
    let [whole] = Target::Whole.cases().unwrap().try_into().unwrap();

    assert!(extract.recording().ends_with(EXTRACT));
    assert!(whole.recording().ends_with(WHOLE));
    assert!(extract.recording().join("video.json").is_file());
}

#[test]
fn a_committed_case_finds_its_video_among_the_fixtures() {
    let video = Case::committed(EXTRACT).video("full_range.mp4").unwrap();

    assert!(video.is_file());
}

#[test]
fn a_recorded_case_finds_no_video_that_is_not_there() {
    let case = Case::recorded(PathBuf::from("data/video_000"));

    assert_eq!(case.recording(), PathBuf::from("data/video_000"));
    assert!(case.video("no_such_video.mp4").is_err());
}
