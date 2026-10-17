use std::fs;

use super::{RefusalRecord, RefusedInput, clear_refusals, read_refusals, record_refusals, refusals_path};

fn record(paths: &[&str]) -> RefusalRecord {
    RefusalRecord {
        date: "2026-10-16T19:02:00Z".to_owned(),
        inputs: paths
            .iter()
            .map(|path| RefusedInput {
                path: (*path).to_owned(),
                reason: "missing".to_owned(),
                cause: None,
                todo: "input-missing".to_owned(),
                videos: Vec::new(),
                detail: None,
            })
            .collect(),
    }
}

#[test]
fn recorded_refusals_are_read_back_whole() {
    let root = tempfile::tempdir().unwrap();
    let written = RefusalRecord {
        inputs: vec![RefusedInput {
            path: "/nuit/video_093.mp4".to_owned(),
            reason: "unopened".to_owned(),
            cause: Some("damaged".to_owned()),
            todo: "damaged".to_owned(),
            videos: vec!["/nuit/a.mp4".to_owned()],
            detail: Some("moov atom not found".to_owned()),
        }],
        ..record(&[])
    };
    record_refusals(root.path(), &written).unwrap();
    assert_eq!(read_refusals(root.path()), Some(written));
}

#[test]
fn the_refusals_sit_beside_the_failures_in_the_hidden_folder() {
    let root = tempfile::tempdir().unwrap();
    record_refusals(root.path(), &record(&["/nuit/v.mp4"])).unwrap();
    assert!(root.path().join(".batdetect/refus.json").is_file());
    assert_eq!(refusals_path(root.path()), root.path().join(".batdetect/refus.json"));
}

#[test]
fn a_later_launch_replaces_the_refusals_of_the_earlier_one() {
    let root = tempfile::tempdir().unwrap();
    record_refusals(root.path(), &record(&["/nuit/a.mp4", "/nuit/b.mp4"])).unwrap();
    record_refusals(root.path(), &record(&["/nuit/c.mp4"])).unwrap();
    assert_eq!(read_refusals(root.path()), Some(record(&["/nuit/c.mp4"])));
}

#[test]
fn clearing_removes_the_refusals() {
    let root = tempfile::tempdir().unwrap();
    record_refusals(root.path(), &record(&["/nuit/v.mp4"])).unwrap();
    clear_refusals(root.path()).unwrap();
    assert_eq!(read_refusals(root.path()), None);
}

#[test]
fn clearing_where_nothing_was_refused_is_not_an_error() {
    let root = tempfile::tempdir().unwrap();
    clear_refusals(root.path()).unwrap();
    assert!(!root.path().join(".batdetect").exists());
}

#[test]
fn an_unreadable_file_reads_as_no_refusal() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join(".batdetect")).unwrap();
    fs::write(root.path().join(".batdetect/refus.json"), "{ pas du json").unwrap();
    assert_eq!(read_refusals(root.path()), None);
}

#[test]
fn a_record_without_any_input_reads_as_no_refusal() {
    let root = tempfile::tempdir().unwrap();
    record_refusals(root.path(), &record(&[])).unwrap();
    assert_eq!(read_refusals(root.path()), None);
}
