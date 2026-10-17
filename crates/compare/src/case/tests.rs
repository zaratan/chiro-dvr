use std::path::PathBuf;

use reference::location::Reference;
use reference::recording::SUMMARY_FILE;

use super::{Case, EXTRACT, Target, WHOLE, WHOLE_AT_1440};

#[test]
fn the_extract_and_the_whole_092_are_read_from_the_committed_reference() {
    let [extract] = Target::Extract.cases(Reference::V011).unwrap().try_into().unwrap();
    let [whole] = Target::Whole.cases(Reference::V011).unwrap().try_into().unwrap();

    assert!(extract.recording().ends_with(EXTRACT));
    assert!(whole.recording().ends_with(WHOLE));
    assert!(extract.recording().join("video.json").is_file());
}

#[test]
fn a_committed_case_finds_its_video_among_the_fixtures() {
    let video = Case::committed(Reference::V011, EXTRACT)
        .video("full_range.mp4")
        .unwrap();

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
    let cases = Target::Settings.cases(Reference::V011).unwrap();

    assert_eq!(cases.len(), 16);
    assert!(cases.iter().all(|case| case.recording().join(SUMMARY_FILE).is_file()));
    assert!(cases[0].recording().ends_with("settings/max_area_400"));
}

#[test]
fn a_setting_case_is_named_by_its_folder_since_its_video_is_shared_with_other_cases() {
    let setting = Case::setting(PathBuf::from("fixtures/reference/settings/threshold_25"));
    let extract = Case::committed(Reference::V011, EXTRACT);

    assert_eq!(setting.name("video_092_original_3m24-4m05.mp4"), "threshold_25");
    assert_eq!(
        extract.name("video_092_original_3m24-4m05.mp4"),
        "video_092_original_3m24-4m05.mp4"
    );
    assert!(setting.video("full_range.mp4").unwrap().is_file());
}

#[test]
fn the_092_worked_at_1440_pixels_is_named_by_its_folder_so_as_not_to_overwrite_the_092() {
    let [wide] = Target::WholeAt1440.cases(Reference::V011).unwrap().try_into().unwrap();

    assert!(wide.recording().ends_with(WHOLE_AT_1440));
    assert_eq!(wide.name("video_092_original.mp4"), WHOLE_AT_1440);
}

#[test]
fn each_case_lands_in_the_folder_it_has_under_its_reference_so_a_new_reference_keeps_the_same_tree() {
    let folders: Vec<PathBuf> = [Target::Extract, Target::Whole, Target::WholeAt1440, Target::Settings]
        .into_iter()
        .flat_map(|target| target.cases(Reference::V011).unwrap())
        .map(|case| case.folder())
        .chain([Case::recorded(PathBuf::from("/somewhere/data/video_120")).folder()])
        .collect();

    assert_eq!(folders.len(), 20);
    assert_eq!(
        folders[..4],
        [
            PathBuf::from(EXTRACT),
            PathBuf::from(WHOLE),
            PathBuf::from(WHOLE_AT_1440),
            PathBuf::from("settings/max_area_400"),
        ]
    );
    assert_eq!(folders[19], PathBuf::from("data/video_120"));
}
