use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroUsize;
use std::path::Path;

use types::frame::Pts;
use types::rational::Rational;

use super::{Placed, microseconds};
use crate::decode::{Decoder, Threads};
use crate::frame::{ColorRange, YuvFrame};
use crate::testing;

fn one_thread(path: &Path) -> Decoder {
    Decoder::open(path, Threads::Exactly(NonZeroUsize::MIN)).unwrap()
}

fn fingerprint(frame: &YuvFrame) -> u64 {
    let mut hasher = DefaultHasher::new();
    (frame.y(), frame.u(), frame.v()).hash(&mut hasher);
    hasher.finish()
}

fn sequential(path: &Path) -> Vec<(Pts, u64)> {
    let mut decoder = one_thread(path);
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut frames = Vec::new();
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        frames.push((info.pts().unwrap(), fingerprint(&frame)));
    }
    frames
}

fn served(mut placed: Placed) -> Vec<(Pts, u64)> {
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut frames = Vec::new();
    while let Some(placed_frame) = placed.next(&mut frame).unwrap() {
        frames.push((placed_frame.pts().unwrap(), fingerprint(&frame)));
    }
    frames
}

#[test]
fn a_seek_between_key_frames_lands_on_the_previous_one_with_the_pixels_of_a_reading_from_the_start() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode_with_keys(folder.path(), 25, 10);
    let all = sequential(&path);
    let placed = one_thread(&path).seek_before(all[15].0).unwrap();
    assert_eq!(placed.stream().width(), testing::WIDTH);
    assert_eq!(served(placed), all[10..]);
}

#[test]
fn a_seek_to_a_key_frame_lands_on_it() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode_with_keys(folder.path(), 25, 10);
    let all = sequential(&path);
    assert_eq!(served(one_thread(&path).seek_before(all[20].0).unwrap()), all[20..]);
}

#[test]
fn a_decoder_read_to_its_end_seeks_back_and_serves_again() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode_with_keys(folder.path(), 25, 10);
    let all = sequential(&path);
    let mut decoder = one_thread(&path);
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    while decoder.next(&mut frame).unwrap().is_some() {}
    assert_eq!(served(decoder.seek_before(all[12].0).unwrap()), all[10..]);
}

#[test]
fn a_decoder_placed_at_its_start_serves_every_frame_by_its_pts() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode_with_keys(folder.path(), 25, 10);
    assert_eq!(served(one_thread(&path).at_start()), sequential(&path));
}

#[test]
fn the_seek_target_rounds_down_to_the_microsecond_so_it_never_passes_the_frame() {
    let ninety_khz = Rational::new(1, 90_000).unwrap();
    assert_eq!(microseconds(Pts::new(3), ninety_khz), 33);
    assert_eq!(microseconds(Pts::new(-3), ninety_khz), -34);
    assert_eq!(microseconds(Pts::new(3003), Rational::new(1, 30_000).unwrap()), 100_100);
}

#[test]
fn a_seek_target_beyond_sixty_four_bits_of_microseconds_saturates() {
    let seconds = Rational::new(1, 1).unwrap();
    assert_eq!(microseconds(Pts::new(i64::MAX), seconds), i64::MAX);
    assert_eq!(microseconds(Pts::new(i64::MIN), seconds), i64::MIN);
}
