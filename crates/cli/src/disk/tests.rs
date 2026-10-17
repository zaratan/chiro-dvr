use std::fs;

use super::volume;

#[cfg(unix)]
#[test]
fn a_folder_still_to_create_reads_the_free_space_of_the_volume_of_its_nearest_existing_folder() {
    let root = tempfile::tempdir().unwrap();
    let existing = volume(root.path()).unwrap();
    let missing = volume(&root.path().join("nuit").join("batdetect")).unwrap();
    assert_eq!(missing.device, existing.device);
    assert!(existing.free > 0);
}

#[cfg(unix)]
#[test]
fn two_folders_of_one_volume_share_its_device() {
    let root = tempfile::tempdir().unwrap();
    let (first, second) = (root.path().join("a"), root.path().join("b"));
    fs::create_dir(&first).unwrap();
    fs::create_dir(&second).unwrap();
    assert_eq!(volume(&first).unwrap().device, volume(&second).unwrap().device);
}

#[cfg(not(unix))]
#[test]
fn outside_macos_and_linux_the_free_space_is_not_read() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("a")).unwrap();
    assert_eq!(volume(&root.path().join("a")), None);
}
