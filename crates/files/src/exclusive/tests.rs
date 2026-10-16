use std::fs;
use std::io;
use std::path::Path;

use tempfile::TempPath;

use super::{rename, reserved_then_renamed};

const TABLE: &str = "track,verdict\r\n1,\r\n";
const ANNOTATED: &str = "track,verdict\r\n1,yes\r\n";
const PARTIAL: &str = ".v_tracks.csv.partial";
const FINAL: &str = "v_tracks.csv";

fn left_in(dest: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dest)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

fn written_partial(dest: &Path) -> TempPath {
    let path = dest.join(PARTIAL);
    fs::write(&path, TABLE).unwrap();
    TempPath::try_from_path(path).unwrap()
}

fn missing_partial(dest: &Path) -> TempPath {
    TempPath::try_from_path(dest.join(PARTIAL)).unwrap()
}

#[test]
fn the_fallback_reserves_the_name_then_puts_the_whole_partial_under_it_and_leaves_no_partial() {
    let dest = tempfile::tempdir().unwrap();
    let path = dest.path().join(FINAL);
    reserved_then_renamed(written_partial(dest.path()), &path).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), TABLE);
    assert_eq!(left_in(dest.path()), [FINAL]);
}

#[test]
fn the_fallback_never_writes_over_a_file_already_there_and_drops_its_partial() {
    let dest = tempfile::tempdir().unwrap();
    let path = dest.path().join(FINAL);
    fs::write(&path, ANNOTATED).unwrap();
    let error = reserved_then_renamed(written_partial(dest.path()), &path).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists, "{error}");
    assert_eq!(fs::read_to_string(&path).unwrap(), ANNOTATED);
    assert_eq!(left_in(dest.path()), [FINAL]);
}

#[test]
fn a_fallback_rename_that_fails_leaves_no_empty_reservation_behind() {
    let dest = tempfile::tempdir().unwrap();
    let error = reserved_then_renamed(missing_partial(dest.path()), &dest.path().join(FINAL)).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound, "{error}");
    assert_eq!(left_in(dest.path()), Vec::<String>::new());
}

#[test]
fn a_rename_onto_a_name_already_taken_is_refused_and_writes_nothing_over_it() {
    let dest = tempfile::tempdir().unwrap();
    let path = dest.path().join(FINAL);
    fs::write(&path, ANNOTATED).unwrap();
    let error = rename(written_partial(dest.path()), &path).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists, "{error}");
    assert_eq!(fs::read_to_string(&path).unwrap(), ANNOTATED);
    assert_eq!(left_in(dest.path()), [FINAL]);
}

#[test]
fn a_rename_that_fails_for_another_reason_than_a_taken_name_is_reported_and_leaves_nothing_behind() {
    let dest = tempfile::tempdir().unwrap();
    let error = rename(missing_partial(dest.path()), &dest.path().join(FINAL)).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound, "{error}");
    assert_eq!(left_in(dest.path()), Vec::<String>::new());
}
