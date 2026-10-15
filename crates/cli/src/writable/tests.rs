use std::fs;
use std::path::{Path, PathBuf};

use super::{nearest_existing, unwritable_folders};
use crate::access::Access;
use crate::jobs::Job;

fn job(dest: PathBuf) -> Job {
    Job {
        video: PathBuf::from("v.mp4"),
        dest,
    }
}

#[test]
fn a_missing_folder_is_probed_through_its_nearest_existing_folder_and_nothing_is_left() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("nuit").join("batdetect");
    let refused = unwritable_folders(&[job(folder.join("v")), job(folder.join("w"))]);
    assert!(refused.is_empty(), "{refused:?}");
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn a_relative_folder_that_does_not_exist_yet_is_probed_from_the_current_folder() {
    assert_eq!(
        nearest_existing(Path::new("absent-folder/batdetect")),
        Some(Path::new("."))
    );
}

#[test]
fn a_file_in_place_of_the_folder_is_refused_by_name() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("out");
    fs::write(&file, "").unwrap();
    let refused = unwritable_folders(&[job(file.join("v"))]);
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].folder, file);
    assert_eq!(refused[0].access, Access::NotAFolder);
}

#[cfg(unix)]
#[test]
fn an_existing_folder_without_write_permission_is_refused_and_each_refused_folder_is_named() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let locked = [root.path().join("a"), root.path().join("b")];
    let open = root.path().join("c");
    for folder in locked.iter().chain([&open]) {
        fs::create_dir(folder).unwrap();
    }
    for folder in &locked {
        fs::set_permissions(folder, fs::Permissions::from_mode(0o555)).unwrap();
    }
    let jobs: Vec<Job> = locked
        .iter()
        .chain([&open])
        .map(|folder| job(folder.join("v")))
        .collect();
    let refused = unwritable_folders(&jobs);
    for folder in &locked {
        fs::set_permissions(folder, fs::Permissions::from_mode(0o755)).unwrap();
    }
    assert!(
        refused.iter().all(|refusal| refusal.access == Access::Refused),
        "{refused:?}"
    );
    let folders: Vec<PathBuf> = refused.into_iter().map(|refusal| refusal.folder).collect();
    assert_eq!(folders, locked);
}
