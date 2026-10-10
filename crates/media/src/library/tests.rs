use std::fs;
use std::path::Path;

use super::{license, version};

fn repository() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn the_linked_ffmpeg_is_the_build_dot_ffmpeg_points_to() {
    let versions = fs::read_to_string(repository().join("packaging/ffmpeg-versions")).unwrap();
    let pinned = versions
        .lines()
        .find_map(|line| line.strip_prefix("FFMPEG_VERSION="))
        .unwrap();
    let target = fs::read_link(repository().join(".ffmpeg")).unwrap();
    let fingerprint = target.file_name().unwrap().to_str().unwrap().split('-').next().unwrap();
    assert_eq!(version(), format!("{pinned}-{fingerprint}"));
}

#[test]
fn the_linked_ffmpeg_is_lgpl() {
    assert_eq!(license(), "LGPL version 2.1 or later");
}
