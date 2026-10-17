use std::io;

use super::free_bytes;

#[cfg(unix)]
#[test]
fn the_free_space_of_a_folder_is_that_of_its_volume_and_is_not_zero_on_a_working_disk() {
    let folder = tempfile::tempdir().unwrap();
    assert!(free_bytes(folder.path()).unwrap() > 0);
}

#[cfg(unix)]
#[test]
fn a_missing_folder_has_no_free_space_to_report() {
    let folder = tempfile::tempdir().unwrap();
    let error = free_bytes(&folder.path().join("absent")).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
}

#[cfg(not(unix))]
#[test]
fn the_free_space_is_not_measured_outside_unix() {
    let folder = tempfile::tempdir().unwrap();
    let error = free_bytes(folder.path()).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Unsupported);
}
