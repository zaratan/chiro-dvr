use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroUsize;
use std::path::Path;

use types::rational::Rational;

use super::Fanout;
use crate::decode::{Decoder, Threads};
use crate::encode::{Codec, Error, Settings};
use crate::frame::{ColorRange, YuvFrame};
use crate::testing::{HEIGHT, WIDTH, pattern};

fn fanout() -> Fanout {
    let settings = Settings::new(WIDTH, HEIGHT, Rational::new(30, 1).unwrap(), Codec::OpenH264).unwrap();
    Fanout::create(&settings).unwrap()
}

fn read(path: &Path) -> Vec<(bool, i64, u64)> {
    let mut decoder = Decoder::open(path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut frames = Vec::new();
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        let mut hasher = DefaultHasher::new();
        (frame.y(), frame.u(), frame.v()).hash(&mut hasher);
        frames.push((info.is_key(), info.pts().unwrap().get(), hasher.finish()));
    }
    frames
}

fn names_in(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn overlapping_clips_cut_from_one_encoding_hold_their_frames_and_share_identical_pixels() {
    let folder = tempfile::tempdir().unwrap();
    let (first, second) = (folder.path().join("first.mp4"), folder.path().join("second.mp4"));
    let mut fanout = fanout();
    for index in 0..12_u8 {
        if index == 0 {
            fanout.open(&first, 10).unwrap();
        }
        if index == 4 {
            fanout.open(&second, 8).unwrap();
        }
        fanout.push(&pattern(index)).unwrap();
    }
    fanout.finish().unwrap();
    let (early, late) = (read(&first), read(&second));
    assert_eq!((early.len(), late.len()), (10, 8));
    assert!(early[0].0 && late[0].0);
    assert_eq!((early[0].1, late[0].1), (0, 0));
    let shared: Vec<u64> = early[4..].iter().map(|frame| frame.2).collect();
    let again: Vec<u64> = late[..6].iter().map(|frame| frame.2).collect();
    assert_eq!(shared, again);
    assert_eq!(names_in(folder.path()), ["first.mp4", "second.mp4"]);
}

#[test]
fn a_clip_of_no_frame_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("empty.mp4");
    assert!(matches!(fanout().open(&path, 0), Err(Error::EmptyClip { .. })));
}

#[test]
fn a_clip_still_waiting_for_frames_at_the_end_fails_and_leaves_no_file_while_the_whole_one_stays() {
    let folder = tempfile::tempdir().unwrap();
    let (whole, short) = (folder.path().join("whole.mp4"), folder.path().join("short.mp4"));
    let mut fanout = fanout();
    fanout.open(&whole, 3).unwrap();
    fanout.open(&short, 5).unwrap();
    for index in 0..3 {
        fanout.push(&pattern(index)).unwrap();
    }
    assert!(matches!(fanout.finish(), Err(Error::ShortClip { path }) if path == short));
    assert_eq!(names_in(folder.path()), ["whole.mp4"]);
    assert_eq!(read(&whole).len(), 3);
}

#[test]
fn a_clip_in_a_missing_folder_is_a_destination_error() {
    let folder = tempfile::tempdir().unwrap();
    let error = fanout()
        .open(&folder.path().join("missing").join("clip.mp4"), 3)
        .unwrap_err();
    assert!(error.concerns_the_destination(), "{error:?}");
}
