use std::fs;

use types::rational::Rational;

use super::{Codec, Encoder, Error, Settings};
use crate::decode::Decoder;
use crate::frame::{ColorRange, YuvFrame};
use crate::testing::{self, HEIGHT, WIDTH};

fn thirty() -> Rational {
    Rational::new(30, 1).unwrap()
}

fn decoded_frames(path: &std::path::Path) -> (u32, u32, u32) {
    let mut decoder = Decoder::open(path).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut count = 0;
    while decoder.next(&mut frame).unwrap().is_some() {
        count += 1;
    }
    (count, frame.width(), frame.height())
}

#[test]
fn one_second_of_openh264_reads_back_as_30_frames_of_the_same_size() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 30);
    assert_eq!(decoded_frames(&path), (30, WIDTH, HEIGHT));
}

#[test]
fn an_odd_size_is_refused_before_encoding() {
    assert!(matches!(
        Settings::new(321, 240, thirty(), Codec::OpenH264),
        Err(Error::Size {
            width: 321,
            height: 240
        })
    ));
    assert!(matches!(
        Settings::new(0, 240, thirty(), Codec::OpenH264),
        Err(Error::Size { .. })
    ));
}

#[test]
fn a_rate_that_is_not_positive_is_refused() {
    let backwards = Rational::new(-30, 1).unwrap();
    assert!(matches!(
        Settings::new(320, 240, backwards, Codec::OpenH264),
        Err(Error::FrameRate { .. })
    ));
}

#[test]
fn a_frame_of_another_size_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&folder.path().join("clip.mp4"), &settings).unwrap();
    let other = YuvFrame::new(160, 120, ColorRange::Limited).unwrap();
    assert!(matches!(
        encoder.push(&other),
        Err(Error::FrameSize { found_width: 160, .. })
    ));
}

#[test]
fn an_unfinished_encoder_leaves_no_file_behind() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&folder.path().join("clip.mp4"), &settings).unwrap();
    encoder.push(&testing::pattern(0)).unwrap();
    assert!(format!("{encoder:?}").contains("clip.mp4"));
    drop(encoder);
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
}

#[test]
fn a_destination_in_a_missing_folder_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let path = folder.path().join("absent/clip.mp4");
    assert!(matches!(Encoder::create(&path, &settings), Err(Error::Create { .. })));
}

#[test]
fn a_destination_ffmpeg_cannot_take_is_refused() {
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let path = std::path::Path::new("cl\0ip.mp4");
    assert!(matches!(
        Encoder::create(path, &settings),
        Err(Error::UnusablePath { .. })
    ));
}

#[cfg(target_os = "macos")]
#[test]
fn videotoolbox_with_software_fallback_encodes_one_second() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::VideoToolbox { allow_software: true }, 30);
    assert_eq!(decoded_frames(&path), (30, WIDTH, HEIGHT));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn videotoolbox_is_missing_outside_macos() {
    let codec = Codec::VideoToolbox { allow_software: true };
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), codec).unwrap();
    let folder = tempfile::tempdir().unwrap();
    assert!(matches!(
        Encoder::create(&folder.path().join("clip.mp4"), &settings),
        Err(Error::EncoderMissing {
            name: "h264_videotoolbox"
        })
    ));
}

#[test]
fn a_size_ffmpeg_cannot_encode_is_refused_when_the_encoder_starts() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(16_384, 16_384, thirty(), Codec::OpenH264).unwrap();
    let error = Encoder::create(&folder.path().join("huge.mp4"), &settings).unwrap_err();
    assert!(matches!(
        error,
        Error::Open {
            name: "libopenh264",
            ..
        }
    ));
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
}

#[test]
fn a_destination_taken_by_a_folder_fails_at_finish_without_losing_the_folder() {
    let folder = tempfile::tempdir().unwrap();
    let taken = folder.path().join("clip.mp4");
    fs::create_dir(&taken).unwrap();
    fs::write(taken.join("keep.txt"), "kept").unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&taken, &settings).unwrap();
    encoder.push(&testing::pattern(0)).unwrap();
    assert!(matches!(encoder.finish(), Err(Error::Persist { .. })));
    assert_eq!(fs::read_to_string(taken.join("keep.txt")).unwrap(), "kept");
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 1);
}

#[test]
fn a_full_range_frame_is_refused_rather_than_written_as_limited_range() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&folder.path().join("clip.mp4"), &settings).unwrap();
    let full = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Full).unwrap();
    assert!(matches!(encoder.push(&full), Err(Error::FullRange)));
}
