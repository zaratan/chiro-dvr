use std::fs;
use std::path::{Path, PathBuf};

use files::refusals::{RefusalRecord, RefusedInput, read_refusals, record_refusals};
use jiff::Timestamp;

use super::Refusals;
use crate::access::Access;
use crate::failure::Cause;
use crate::jobs::{Reason, Refused};

const NOW: &str = "2026-10-16T19:02:00Z";

fn now() -> Timestamp {
    NOW.parse().unwrap()
}

fn refused(path: &str, reason: Reason) -> Refused {
    Refused {
        path: PathBuf::from(path),
        reason,
    }
}

fn written(root: &Path) -> Vec<RefusedInput> {
    read_refusals(root).map(|record| record.inputs).unwrap_or_default()
}

#[test]
fn the_refusals_of_the_launch_are_written_with_their_reason_and_what_to_do() {
    let root = tempfile::tempdir().unwrap();
    let inputs = [
        refused("/nuit/absente.mp4", Reason::Missing),
        refused("/nuit/fermee.mp4", Reason::Unreadable(Access::Refused)),
    ];
    let mut refusals = Refusals::new(vec![root.path().to_path_buf()], &inputs, now());
    assert!(refusals.note().is_empty());
    let record = read_refusals(root.path()).unwrap();
    assert_eq!(record.date, NOW);
    assert_eq!(
        record
            .inputs
            .iter()
            .map(|input| (input.path.as_str(), input.reason.as_str(), input.todo.as_str()))
            .collect::<Vec<_>>(),
        [
            ("/nuit/absente.mp4", "missing", "input-missing"),
            ("/nuit/fermee.mp4", "refused", "input-read-refused"),
        ]
    );
}

fn unopened(cause: Cause) -> Refused {
    refused(
        "/nuit/video_093.mp4",
        Reason::Unopened {
            cause,
            detail: "codec hevc".to_owned(),
        },
    )
}

#[test]
fn a_video_that_did_not_open_keeps_its_cause_and_the_detail_its_advice_refers_to() {
    let root = tempfile::tempdir().unwrap();
    Refusals::new(vec![root.path().to_path_buf()], &[unopened(Cause::Unsupported)], now()).note();
    let [input] = written(root.path()).try_into().unwrap();
    assert_eq!(input.reason, "unopened");
    assert_eq!(input.cause.as_deref(), Some("unsupported"));
    assert_eq!(input.todo, "unsupported");
    assert_eq!(input.detail.as_deref(), Some("codec hevc"));
}

#[test]
fn the_detail_is_left_out_when_the_advice_does_not_refer_to_it() {
    let root = tempfile::tempdir().unwrap();
    Refusals::new(vec![root.path().to_path_buf()], &[unopened(Cause::Damaged)], now()).note();
    let [input] = written(root.path()).try_into().unwrap();
    assert_eq!(input.detail, None);
}

#[test]
fn a_shared_results_folder_names_the_videos_that_share_it() {
    let root = tempfile::tempdir().unwrap();
    let inputs = [refused(
        "/nuit/batdetect/v",
        Reason::SharedFolder {
            videos: vec![PathBuf::from("/nuit/v.mp4"), PathBuf::from("/nuit/V.mp4")],
        },
    )];
    Refusals::new(vec![root.path().to_path_buf()], &inputs, now()).note();
    let [input] = written(root.path()).try_into().unwrap();
    assert_eq!(input.videos, ["/nuit/v.mp4", "/nuit/V.mp4"]);
}

#[test]
fn every_page_of_the_launch_gets_the_same_refusals() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let inputs = [refused("/nuit/absente.mp4", Reason::Missing)];
    let roots = vec![first.path().to_path_buf(), second.path().to_path_buf()];
    Refusals::new(roots, &inputs, now()).note();
    assert_eq!(read_refusals(first.path()), read_refusals(second.path()));
    assert_eq!(written(first.path()).len(), 1);
}

#[test]
fn a_launch_without_refusal_removes_the_refusals_of_the_previous_one() {
    let root = tempfile::tempdir().unwrap();
    let earlier = RefusalRecord {
        date: NOW.to_owned(),
        inputs: written_by_hand(),
    };
    record_refusals(root.path(), &earlier).unwrap();
    Refusals::new(vec![root.path().to_path_buf()], &[], now()).note();
    assert_eq!(read_refusals(root.path()), None);
}

#[test]
fn a_results_folder_created_later_gets_the_refusals_when_it_appears() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("batdetect");
    let inputs = [refused("/nuit/absente.mp4", Reason::Missing)];
    let mut refusals = Refusals::new(vec![root.clone()], &inputs, now());
    assert!(refusals.note().is_empty());
    assert!(!root.exists());
    fs::create_dir(&root).unwrap();
    refusals.note();
    assert_eq!(written(&root).len(), 1);
}

#[test]
fn the_refusals_are_written_once_per_folder() {
    let root = tempfile::tempdir().unwrap();
    let inputs = [refused("/nuit/absente.mp4", Reason::Missing)];
    let mut refusals = Refusals::new(vec![root.path().to_path_buf()], &inputs, now());
    refusals.note();
    record_refusals(
        root.path(),
        &RefusalRecord {
            date: NOW.to_owned(),
            inputs: written_by_hand(),
        },
    )
    .unwrap();
    refusals.note();
    assert_eq!(written(root.path()), written_by_hand());
}

fn written_by_hand() -> Vec<RefusedInput> {
    vec![RefusedInput {
        path: "/autre/nuit.mp4".to_owned(),
        reason: "missing".to_owned(),
        cause: None,
        todo: "input-missing".to_owned(),
        videos: Vec::new(),
        detail: None,
    }]
}
