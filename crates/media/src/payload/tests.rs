use std::fs;
use std::io;

use super::wipe;
use crate::decode::{Decoder, Threads};
use crate::encode::Codec;
use crate::frame::{ColorRange, YuvFrame};
use crate::testing;

fn decoded(path: &std::path::Path) -> usize {
    let mut decoder = Decoder::open(path, Threads::Automatic).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut count = 0;
    while decoder.next(&mut frame).unwrap().is_some() {
        count += 1;
    }
    count
}

#[test]
fn a_wiped_quarter_of_the_payload_loses_frames_but_leaves_the_file_readable() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 30);
    wipe(&path, 2..3, 4).unwrap();
    assert!((1..30).contains(&decoded(&path)));
}

#[test]
fn a_file_without_media_data_is_refused_as_invalid() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("empty.mp4");
    fs::write(&path, b"ftypisom").unwrap();
    assert_eq!(wipe(&path, 0..1, 4).unwrap_err().kind(), io::ErrorKind::InvalidData);
}

#[test]
fn a_media_data_tag_without_its_size_is_refused_as_invalid() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("cut.mp4");
    fs::write(&path, b"\0mdat").unwrap();
    assert_eq!(wipe(&path, 0..1, 4).unwrap_err().kind(), io::ErrorKind::InvalidData);
}

#[test]
fn parts_beyond_the_payload_empty_or_overflowing_are_refused() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 3);
    for (parts, count) in [(4..6, 4), (0..1, 0), (2..2, 4), (0..usize::MAX, 4)] {
        assert_eq!(
            wipe(&path, parts, count).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}

#[test]
fn a_missing_file_is_a_read_error() {
    let folder = tempfile::tempdir().unwrap();
    let missing = folder.path().join("missing.mp4");
    assert_eq!(wipe(&missing, 0..1, 4).unwrap_err().kind(), io::ErrorKind::NotFound);
}
