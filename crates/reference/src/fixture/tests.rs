use std::fs;

use super::{Error, checked, fixture, fixtures};

#[test]
fn a_git_lfs_pointer_is_refused_with_a_pull_hint() {
    let directory = tempfile::tempdir().unwrap();
    let pointer = directory.path().join("video.mp4");
    fs::write(
        &pointer,
        "version https://git-lfs.github.com/spec/v1\noid sha256:00\nsize 60240812\n",
    )
    .unwrap();
    let error = checked(pointer).unwrap_err();
    assert!(matches!(error, Error::LfsPointer { .. }));
    assert!(error.to_string().contains("git lfs pull"));
}

#[test]
fn a_missing_fixture_is_reported_by_name() {
    let error = fixture("absent.mp4").unwrap_err();
    assert!(matches!(error, Error::Unreadable { .. }));
    assert!(error.to_string().contains("absent.mp4"));
}

#[test]
fn a_file_shorter_than_a_pointer_is_not_one() {
    let directory = tempfile::tempdir().unwrap();
    let short = directory.path().join("short.json");
    fs::write(&short, "{}").unwrap();
    assert_eq!(checked(short.clone()).unwrap(), short);
}

#[test]
fn a_real_file_is_handed_back() {
    let directory = tempfile::tempdir().unwrap();
    let video = directory.path().join("video.mp4");
    fs::write(&video, [0_u8; 64]).unwrap();
    assert_eq!(checked(video.clone()).unwrap(), video);
}

#[test]
fn fixtures_resolve_to_the_repository_whatever_the_crate() {
    assert!(fixtures().join("video_092_original_3m24-4m05.mp4").is_file());
}
