use std::path::PathBuf;

use reference::recording::SUMMARY_FILE;

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

#[test]
fn the_setting_cases_are_the_sixteen_committed_recordings_of_the_settings_folder() {
    let cases = Target::Settings.cases().unwrap();

    assert_eq!(cases.len(), 16);
    assert!(cases.iter().all(|case| case.recording().join(SUMMARY_FILE).is_file()));
    assert!(cases[0].recording().ends_with("settings/max_area_400"));
}

#[test]
fn a_setting_case_is_named_by_its_folder_since_its_video_is_shared_with_other_cases() {
    let setting = Case::setting(PathBuf::from("fixtures/reference/settings/threshold_25"));
    let extract = Case::committed(EXTRACT);

    assert_eq!(setting.name("video_092_original_3m24-4m05.mp4"), "threshold_25");
    assert_eq!(
        extract.name("video_092_original_3m24-4m05.mp4"),
        "video_092_original_3m24-4m05.mp4"
    );
    assert!(setting.video("full_range.mp4").unwrap().is_file());
}

#[test]
fn only_the_setting_cases_expect_refusals_since_every_other_video_must_be_compared() {
    assert!(Target::Settings.tolerates_refusals());
    assert!(
        [Target::Extract, Target::Whole, Target::Recorded]
            .iter()
            .all(|target| !target.tolerates_refusals())
    );
}
