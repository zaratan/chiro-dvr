use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use files::failures::{FailureRecord, read_failures, record_failure};
use jiff::Timestamp;

use super::Ledger;
use crate::jobs::Job;
use crate::options::Options;
use crate::outcome::{Failure, Panic};
use crate::process::{self, Extent};

const EXTENT: Extent = Extent {
    length_s: 40.0,
    not_analysed_s: 0.0,
};
const NOW: &str = "2026-10-16T08:19:00Z";

fn job(root: &Path) -> Job {
    named(root, "video_093")
}

fn named(root: &Path, name: &str) -> Job {
    Job {
        video: PathBuf::from(format!("/nuit/{name}.mp4")),
        dest: root.join(name),
    }
}

fn damaged() -> Failure {
    Failure::Panic(Panic {
        message: "damaged".to_owned(),
        location: None,
    })
}

fn now() -> Timestamp {
    NOW.parse().unwrap()
}

fn noted(root: &Path, failure: Option<&Failure>) -> Vec<FailureRecord> {
    assert!(
        Ledger::default()
            .note(&job(root), failure, &Options::default(), now())
            .is_empty()
    );
    read_failures(root).into_iter().map(|(_, record)| record).collect()
}

fn earlier_failure(root: &Path) {
    let record = FailureRecord {
        video: "video_093.mp4".to_owned(),
        cause: "damaged".to_owned(),
        place: String::new(),
        detail: None,
        date: NOW.to_owned(),
    };
    record_failure(root, OsStr::new("video_093"), &record).unwrap();
}

#[test]
fn a_failed_video_leaves_its_cause_place_detail_and_date_for_the_page() {
    let root = tempfile::tempdir().unwrap();
    let error = process::Error::Folder {
        dest: root.path().join("video_093"),
        source: io::Error::from(io::ErrorKind::PermissionDenied),
    };
    let records = noted(root.path(), Some(&Failure::Error(error)));
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].video, "video_093.mp4");
    assert_eq!(records[0].cause, "forbidden");
    assert_eq!(records[0].place, root.path().join("video_093").to_string_lossy());
    assert!(
        records[0]
            .detail
            .as_deref()
            .is_some_and(|detail| detail.starts_with("cannot create"))
    );
    assert_eq!(records[0].date, NOW);
}

#[test]
fn a_panic_is_kept_as_an_internal_error_with_its_location() {
    let root = tempfile::tempdir().unwrap();
    let panic = Panic {
        message: "index out of bounds".to_owned(),
        location: Some("src/track.rs:12:5".to_owned()),
    };
    let records = noted(root.path(), Some(&Failure::Panic(panic)));
    assert_eq!(records[0].cause, "internal");
    assert_eq!(
        records[0].detail.as_deref(),
        Some("index out of bounds (src/track.rs:12:5)")
    );
}

#[test]
fn a_success_removes_the_failure_left_by_an_earlier_run() {
    let root = tempfile::tempdir().unwrap();
    earlier_failure(root.path());
    assert_eq!(noted(root.path(), None), Vec::new());
}

#[test]
fn too_many_tracks_is_a_finished_video_and_removes_an_earlier_failure() {
    let root = tempfile::tempdir().unwrap();
    earlier_failure(root.path());
    let error = process::Error::TooManyTracks {
        count: 3,
        limit: 2,
        annotated: false,
        extent: EXTENT,
        kept: Vec::new(),
    };
    assert_eq!(noted(root.path(), Some(&Failure::Error(error))), Vec::new());
}

#[test]
fn a_failure_alone_in_a_folder_creates_no_results_folder() {
    let videos = tempfile::tempdir().unwrap();
    let root = videos.path().join("batdetect");
    let errors = Ledger::default().note(&job(&root), Some(&damaged()), &Options::default(), now());
    assert!(errors.is_empty());
    assert!(!root.exists());
}

#[test]
fn a_failure_is_kept_once_a_later_video_of_the_batch_creates_the_results_folder() {
    let videos = tempfile::tempdir().unwrap();
    let root = videos.path().join("batdetect");
    let mut ledger = Ledger::default();
    assert!(
        ledger
            .note(&job(&root), Some(&damaged()), &Options::default(), now())
            .is_empty()
    );
    std::fs::create_dir_all(root.join("video_094")).unwrap();
    let done = ledger.note(&named(&root, "video_094"), None, &Options::default(), now());
    assert!(done.is_empty());
    let kept: Vec<String> = read_failures(&root)
        .into_iter()
        .map(|(_, record)| record.video)
        .collect();
    assert_eq!(kept, ["video_093.mp4"]);
}
