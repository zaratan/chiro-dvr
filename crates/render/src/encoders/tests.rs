use std::num::NonZeroU32;

use media::encode::Codec;
use media::frame::{ColorRange, YuvFrame};
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::span::Span;

use super::{Encoders, Ffmpeg};
use crate::clips::Error;
use crate::sequential;
use crate::testing::{Breaking, Fake, thirty};
use crate::windows::{self, HARDWARE_LANES, Window};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;

fn window(track: usize, first: usize, last: usize) -> Window {
    Window {
        track,
        span: Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap(),
    }
}

fn gray() -> YuvFrame {
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    frame.y_mut().fill(100);
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    frame
}

#[test]
fn only_the_first_eight_lanes_may_use_videotoolbox() {
    let codecs = [Codec::VideoToolbox, Codec::OpenH264];
    let encoders = Encoders::new(&Ffmpeg, &codecs, thirty());
    let last_hardware: Vec<Codec> = encoders.for_lane(HARDWARE_LANES - 1).collect();
    assert_eq!(last_hardware, codecs);
    let first_software: Vec<Codec> = encoders.for_lane(HARDWARE_LANES).collect();
    assert_eq!(first_software, [Codec::OpenH264]);
}

#[test]
fn once_videotoolbox_is_set_aside_no_lane_gets_it_and_no_session_is_reserved() {
    let codecs = [Codec::VideoToolbox, Codec::OpenH264];
    let encoders = Encoders::new(&Ffmpeg, &codecs, thirty());
    let nine = windows::groups(&(0..9).map(|track| window(track, 0, 10)).collect::<Vec<_>>());
    assert_eq!(encoders.hardware_sessions(&nine[0]), HARDWARE_LANES);
    encoders.set_hardware_aside();
    let first: Vec<Codec> = encoders.for_lane(0).collect();
    assert_eq!(first, [Codec::OpenH264]);
    assert_eq!(encoders.hardware_sessions(&nine[0]), 0);
}

#[test]
fn openh264_writes_a_key_frame_every_second() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("clip.mp4");
    let encoders = Encoders::new(&Ffmpeg, &[Codec::OpenH264], thirty());
    let (mut encoder, codec) = encoders.open(&path, 0, WIDTH, HEIGHT).unwrap();
    assert_eq!(codec, Codec::OpenH264);
    let frame = gray();
    for _ in 0..70 {
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
    let keys: Vec<usize> = sequential::read(&path).keys.iter().map(|key| key.get()).collect();
    assert_eq!(keys, [0, 30, 60]);
}

#[test]
fn a_rate_under_one_frame_per_second_keys_every_frame() {
    let slow = FrameRate::new(Rational::new(1, 3).unwrap()).unwrap();
    assert_eq!(Encoders::new(&Ffmpeg, &[], slow).key_interval, NonZeroU32::MIN);
}

#[test]
fn a_clip_whose_folder_is_missing_is_a_destination_error() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("missing").join("clip.mp4");
    let encoders = Encoders::new(&Ffmpeg, &[Codec::OpenH264], thirty());
    let error = encoders.open(&path, 0, WIDTH, HEIGHT).unwrap_err();
    assert!(matches!(error, Error::Destination { path: refused, .. } if refused == path));
}

#[test]
fn a_lane_left_without_any_codec_is_refused_without_writing() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("clip.mp4");
    let encoders = Encoders::new(&Ffmpeg, &[Codec::VideoToolbox], thirty());
    let error = encoders.open(&path, HARDWARE_LANES, WIDTH, HEIGHT).unwrap_err();
    assert!(matches!(error, Error::NoEncoder { path: refused } if refused == path));
    assert_eq!(folder.path().read_dir().unwrap().count(), 0);
}

#[test]
fn an_odd_size_refused_by_h264_names_the_clip() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("clip.mp4");
    let encoders = Encoders::new(&Ffmpeg, &[Codec::OpenH264], thirty());
    let error = encoders.open(&path, 0, WIDTH + 1, HEIGHT).unwrap_err();
    assert!(matches!(error, Error::Clip { path: refused, .. } if refused == path));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn without_videotoolbox_the_window_falls_back_to_openh264() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("clip.mp4");
    let encoders = Encoders::new(&Ffmpeg, &[Codec::VideoToolbox, Codec::OpenH264], thirty());
    let (encoder, codec) = encoders.open(&path, 0, WIDTH, HEIGHT).unwrap();
    assert_eq!((encoder.codec(), codec), (Codec::OpenH264, Codec::OpenH264));
}

#[test]
fn a_videotoolbox_that_refuses_to_open_hands_its_window_to_openh264_and_stays_allowed() {
    let folder = tempfile::tempdir().unwrap();
    let fake = Fake::breaking(vec![("refused.mp4", Codec::VideoToolbox, Breaking::Open)]);
    let encoders = Encoders::new(&fake, &[Codec::VideoToolbox, Codec::OpenH264], thirty());
    let (_refused, codec) = encoders
        .open(&folder.path().join("refused.mp4"), 0, WIDTH, HEIGHT)
        .unwrap();
    assert_eq!(codec, Codec::OpenH264);
    let (_next, codec) = encoders
        .open(&folder.path().join("next.mp4"), 1, WIDTH, HEIGHT)
        .unwrap();
    assert_eq!(codec, Codec::VideoToolbox);
}
