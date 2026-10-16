use std::ffi::OsStr;
use std::fs;

use super::{FailureRecord, clear_failure, failures_dir, read_failures, record_failure};

fn record(video: &str) -> FailureRecord {
    FailureRecord {
        video: video.to_owned(),
        cause: "damaged".to_owned(),
        place: "/nuit/batdetect/v".to_owned(),
        detail: Some("the detection failed".to_owned()),
        date: "2026-10-16T08:19:00Z".to_owned(),
    }
}

#[test]
fn a_recorded_failure_is_read_back_under_the_name_of_its_folder() {
    let root = tempfile::tempdir().unwrap();
    record_failure(root.path(), OsStr::new("nuit 12 été"), &record("nuit 12 été.mp4")).unwrap();
    assert_eq!(
        read_failures(root.path()),
        vec![("nuit 12 été".into(), record("nuit 12 été.mp4"))]
    );
}

#[test]
fn the_failures_sit_in_a_hidden_folder_at_the_root_of_the_results() {
    let root = tempfile::tempdir().unwrap();
    record_failure(root.path(), OsStr::new("v"), &record("v.mp4")).unwrap();
    assert!(root.path().join(".batdetect/echecs/v.json").is_file());
    assert_eq!(failures_dir(root.path()), root.path().join(".batdetect/echecs"));
}

#[test]
fn a_second_failure_of_the_same_video_replaces_the_first() {
    let root = tempfile::tempdir().unwrap();
    record_failure(root.path(), OsStr::new("v"), &record("v.mp4")).unwrap();
    let later = FailureRecord {
        cause: "disk-full".to_owned(),
        ..record("v.mp4")
    };
    record_failure(root.path(), OsStr::new("v"), &later).unwrap();
    assert_eq!(read_failures(root.path()), vec![("v".into(), later)]);
}

#[test]
fn clearing_removes_only_the_failure_of_that_video() {
    let root = tempfile::tempdir().unwrap();
    record_failure(root.path(), OsStr::new("a"), &record("a.mp4")).unwrap();
    record_failure(root.path(), OsStr::new("b"), &record("b.mp4")).unwrap();
    clear_failure(root.path(), OsStr::new("a")).unwrap();
    assert_eq!(read_failures(root.path()), vec![("b".into(), record("b.mp4"))]);
}

#[test]
fn clearing_a_video_that_never_failed_is_not_an_error() {
    let root = tempfile::tempdir().unwrap();
    clear_failure(root.path(), OsStr::new("v")).unwrap();
    assert!(!root.path().join(".batdetect").exists());
}

#[test]
fn a_root_without_failures_reads_as_none() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(read_failures(root.path()), Vec::new());
}

#[test]
fn an_unreadable_record_hides_only_itself() {
    let root = tempfile::tempdir().unwrap();
    record_failure(root.path(), OsStr::new("b"), &record("b.mp4")).unwrap();
    fs::write(failures_dir(root.path()).join("a.json"), "{ cut").unwrap();
    fs::write(failures_dir(root.path()).join("notes.txt"), "kept by hand").unwrap();
    assert_eq!(read_failures(root.path()), vec![("b".into(), record("b.mp4"))]);
}

#[test]
fn a_record_gets_the_same_rights_as_any_file_written_beside_it() {
    let root = tempfile::tempdir().unwrap();
    record_failure(root.path(), OsStr::new("v"), &record("v.mp4")).unwrap();
    let reference = failures_dir(root.path()).join("reference.txt");
    fs::write(&reference, "").unwrap();
    assert_eq!(
        fs::metadata(failures_dir(root.path()).join("v.json"))
            .unwrap()
            .permissions(),
        fs::metadata(&reference).unwrap().permissions()
    );
}
