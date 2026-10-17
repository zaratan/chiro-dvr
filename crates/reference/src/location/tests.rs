use std::ffi::OsString;
use std::fs;

use super::{
    REFERENCE_DIR_VARIABLE, Reference, bench_under, committed, outputs_under, recording_directories_under,
    reference_video_under, video_under,
};
use crate::error::Error;
use crate::fixture::fixtures;

#[test]
fn committed_recordings_live_beside_the_fixtures() {
    assert!(
        committed(Reference::V011, "video_092_original_3m24-4m05")
            .join("video.json")
            .is_file()
    );
}

#[test]
fn the_v020_reference_lives_in_its_own_folder_beside_the_v011() {
    assert_eq!(
        committed(Reference::V020, "video_092_original"),
        fixtures().join("reference-v0.2.0").join("video_092_original")
    );
    assert_eq!(
        committed(Reference::V011, "video_092_original"),
        fixtures().join("reference").join("video_092_original")
    );
}

#[test]
fn each_reference_reads_its_recordings_and_benches_from_its_own_variable() {
    assert_eq!(Reference::V011.variable(), "BATDETECT_REFERENCE_DIR");
    assert_eq!(Reference::V020.variable(), "BATDETECT_REFERENCE_V020_DIR");
}

#[test]
fn recordings_are_the_folders_of_data_in_name_order() {
    let root = tempfile::tempdir().unwrap();
    let data = root.path().join("data");
    for name in ["video_130", "video_089", "video_103"] {
        fs::create_dir_all(data.join(name)).unwrap();
    }
    fs::write(data.join("notes.txt"), "").unwrap();
    let names: Vec<_> = recording_directories_under(Some(root.path().into()), REFERENCE_DIR_VARIABLE)
        .unwrap()
        .into_iter()
        .map(|path| path.file_name().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["video_089", "video_103", "video_130"]);
}

#[test]
fn a_reference_folder_without_data_is_reported() {
    let root = tempfile::tempdir().unwrap();
    let error = recording_directories_under(Some(root.path().into()), REFERENCE_DIR_VARIABLE).unwrap_err();
    assert!(matches!(error, Error::Read { .. }));
    assert!(error.to_string().ends_with("data"));
}

#[test]
fn an_unset_or_empty_variable_names_the_variable_instead_of_reading_the_current_folder() {
    for root in [None, Some(OsString::new())] {
        let error = recording_directories_under(root, REFERENCE_DIR_VARIABLE).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the variable BATDETECT_REFERENCE_DIR does not name a folder"
        );
    }
}

#[test]
fn a_recorded_video_is_found_by_name_in_a_folder_whose_path_has_spaces() {
    let home = tempfile::tempdir().unwrap();
    let videos = home.path().join("dossier sans titre").join("img_0000");
    fs::create_dir_all(&videos).unwrap();
    fs::write(videos.join("video_092.mp4"), [0_u8; 64]).unwrap();
    let found = video_under(Some(videos.clone().into()), "video_092.mp4").unwrap();
    assert_eq!(found, videos.join("video_092.mp4"));
}

#[test]
fn a_video_absent_from_the_folder_is_reported_with_its_path() {
    let videos = tempfile::tempdir().unwrap();
    let error = video_under(Some(videos.path().into()), "video_130.mp4").unwrap_err();
    assert!(matches!(error, Error::MissingVideo { .. }));
    assert!(error.to_string().contains("video_130.mp4"));
}

#[test]
fn an_unset_or_empty_videos_variable_names_the_variable() {
    for videos in [None, Some(OsString::new())] {
        let error = video_under(videos, "video_092.mp4").unwrap_err();
        assert_eq!(
            error.to_string(),
            "the variable BATDETECT_VIDEOS_DIR does not name a folder"
        );
    }
}

#[test]
fn a_recorded_bench_lives_under_bench_by_its_name_and_needs_the_variable() {
    let root = tempfile::tempdir().unwrap();

    let found = bench_under(Some(root.path().into()), REFERENCE_DIR_VARIABLE, "transit").unwrap();

    assert_eq!(found, root.path().join("bench").join("transit"));
    assert!(matches!(
        bench_under(None, REFERENCE_DIR_VARIABLE, "transit"),
        Err(Error::UnsetVariable { .. })
    ));
}

#[test]
fn the_recorded_outputs_live_under_outputs_by_language_then_video_and_need_the_variable() {
    let root = tempfile::tempdir().unwrap();

    let found = outputs_under(Some(root.path().into()), "fr", "video_097").unwrap();

    assert_eq!(found, root.path().join("outputs").join("fr").join("video_097"));
    assert!(matches!(
        outputs_under(None, "fr", "video_097"),
        Err(Error::UnsetVariable { .. })
    ));
}

#[test]
fn a_video_of_the_reference_lives_under_videos_and_needs_the_variable_and_the_file() {
    let root = tempfile::tempdir().unwrap();
    let videos = root.path().join("videos");
    fs::create_dir_all(&videos).unwrap();
    fs::write(videos.join("video_089-091.mp4"), [0_u8; 64]).unwrap();

    let found = reference_video_under(Some(root.path().into()), "video_089-091.mp4").unwrap();

    assert_eq!(found, videos.join("video_089-091.mp4"));
    assert!(matches!(
        reference_video_under(Some(root.path().into()), "video_097.mp4"),
        Err(Error::MissingVideo { .. })
    ));
    assert!(matches!(
        reference_video_under(None, "video_089-091.mp4"),
        Err(Error::UnsetVariable { .. })
    ));
}
