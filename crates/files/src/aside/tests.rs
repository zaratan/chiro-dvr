use std::fs;
use std::path::Path;

use jiff::civil::date;

use super::{Stamp, set_aside};

const ANNOTATED: &[u8] = b"\xEF\xBB\xBFpiste;verdict\r\n1;oui\r\n";

fn half_past_two() -> Stamp {
    Stamp::at(date(2026, 10, 16).at(14, 30, 0, 0))
}

fn left_in(dest: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dest)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn a_table_set_aside_takes_the_date_and_minute_of_the_run_and_keeps_its_bytes() {
    let dest = tempfile::tempdir().unwrap();
    let table = dest.path().join("v_pistes.csv");
    fs::write(&table, ANNOTATED).unwrap();
    let dated = set_aside(&table, half_past_two()).unwrap();
    assert_eq!(dated, dest.path().join("v_pistes_2026-10-16_1430.csv"));
    assert_eq!(fs::read(&dated).unwrap(), ANNOTATED);
    assert_eq!(left_in(dest.path()), ["v_pistes_2026-10-16_1430.csv"]);
}

#[test]
fn a_second_table_set_aside_in_the_same_minute_takes_a_rank_and_overwrites_nothing() {
    let dest = tempfile::tempdir().unwrap();
    let table = dest.path().join("v_tracks.csv");
    fs::write(dest.path().join("v_tracks_2026-10-16_1430.csv"), "first").unwrap();
    fs::write(&table, ANNOTATED).unwrap();
    let dated = set_aside(&table, half_past_two()).unwrap();
    assert_eq!(dated, dest.path().join("v_tracks_2026-10-16_1430_2.csv"));
    assert_eq!(fs::read(&dated).unwrap(), ANNOTATED);
    assert_eq!(
        fs::read_to_string(dest.path().join("v_tracks_2026-10-16_1430.csv")).unwrap(),
        "first"
    );
}

#[test]
fn the_old_name_of_v0_1_1_is_dated_before_its_extension() {
    let dest = tempfile::tempdir().unwrap();
    let table = dest.path().join("v.tracks.csv");
    fs::write(&table, ANNOTATED).unwrap();
    let dated = set_aside(&table, half_past_two()).unwrap();
    assert_eq!(dated, dest.path().join("v.tracks_2026-10-16_1430.csv"));
}

#[test]
fn a_rename_that_fails_leaves_no_empty_dated_file_behind() {
    let dest = tempfile::tempdir().unwrap();
    let error = set_aside(&dest.path().join("v_pistes.csv"), half_past_two()).unwrap_err();
    assert!(matches!(error, crate::error::Error::SetAside { .. }), "{error}");
    assert_eq!(left_in(dest.path()), Vec::<String>::new());
}

#[cfg(unix)]
#[test]
fn a_table_that_cannot_be_moved_stays_where_it_was() {
    use std::os::unix::fs::PermissionsExt;

    use crate::error::Error;
    let dest = tempfile::tempdir().unwrap();
    let table = dest.path().join("v_pistes.csv");
    fs::write(&table, ANNOTATED).unwrap();
    fs::set_permissions(dest.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let error = set_aside(&table, half_past_two()).unwrap_err();
    fs::set_permissions(dest.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        matches!(&error, Error::SetAside { path, .. } if *path == table),
        "{error}"
    );
    assert_eq!(fs::read(&table).unwrap(), ANNOTATED);
    assert_eq!(left_in(dest.path()), ["v_pistes.csv"]);
}
