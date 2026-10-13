use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;

use types::frame::Pts;
use types::rational::Rational;

use super::Fanout;
use crate::decode::{Decoder, Threads};
use crate::encode::{Codec, Error, Settings, Stamps, StampsError};
use crate::fingerprint;
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
        frames.push((info.is_key(), info.pts().unwrap().get(), fingerprint::of(&frame)));
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

fn ten_thousandth() -> Rational {
    Rational::new(1, 10_000).unwrap()
}

fn stamped(ticks: &[i64]) -> Vec<Pts> {
    ticks.iter().copied().map(Pts::new).collect()
}

fn written_stamps(ticks: &[i64]) -> (Vec<i64>, Rational, Option<i64>) {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("stamped.mp4");
    let mut fanout = fanout();
    fanout
        .open_stamped(&path, Stamps::new(&stamped(ticks), ten_thousandth()).unwrap())
        .unwrap();
    for index in 0..ticks.len() {
        fanout.push(&pattern(index as u8)).unwrap();
    }
    fanout.finish().unwrap();
    let mut decoder = Decoder::open(&path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    let (time_base, duration) = (decoder.stream().time_base(), decoder.stream().duration());
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut pts = Vec::new();
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        pts.push(info.pts().unwrap().get());
    }
    (pts, time_base, duration)
}

#[test]
fn a_stamped_clip_keeps_the_source_timestamps_across_a_jump_and_its_last_frame_lasts_one_frame() {
    let ticks = [0, 333, 666, 2997, 3330, 3663];
    let (pts, time_base, duration) = written_stamps(&ticks);
    assert_eq!(time_base, ten_thousandth());
    assert_eq!(pts, ticks);
    assert_eq!(duration, Some(3663 + 333));
}

#[test]
fn a_stamped_clip_that_starts_after_zero_keeps_its_first_timestamp() {
    let ticks = [3330, 3663, 3996];
    let (pts, _, _) = written_stamps(&ticks);
    assert_eq!(pts, ticks);
}

#[test]
fn timestamps_that_are_empty_negative_or_not_increasing_are_refused() {
    let base = ten_thousandth();
    assert_eq!(Stamps::new(&[], base), Err(StampsError::Empty));
    assert_eq!(
        Stamps::new(&stamped(&[-1, 333]), base),
        Err(StampsError::Negative { tick: -1 })
    );
    assert_eq!(
        Stamps::new(&stamped(&[0, 333, 333]), base),
        Err(StampsError::NotIncreasing { before: 333, tick: 333 })
    );
    let backwards = Rational::new(-1, 10_000).unwrap();
    assert_eq!(
        Stamps::new(&stamped(&[0]), backwards),
        Err(StampsError::TimeBase { time_base: backwards })
    );
}

#[test]
fn a_single_stamped_frame_lasts_one_frame_of_the_encoder_rate() {
    let (_, _, duration) = written_stamps(&[0]);
    assert_eq!(duration, Some(333));
}
