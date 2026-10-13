use std::num::NonZeroU32;

use media::encode::Codec;
use media::frame::{ColorRange, YuvFrame};
use types::rate::FrameRate;
use types::rational::Rational;

use super::{Encoders, Ffmpeg};
use crate::clips::Error;
use crate::sequential;
use crate::testing::{Fake, thirty};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;

fn gray() -> YuvFrame {
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    frame.y_mut().fill(100);
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    frame
}

#[test]
fn a_group_reserves_one_videotoolbox_session_until_videotoolbox_is_set_aside() {
    let encoders = Encoders::new(&Ffmpeg, &[Codec::VideoToolbox, Codec::OpenH264], thirty());
    assert_eq!(encoders.hardware_sessions(), 1);
    encoders.set_hardware_aside();
    assert_eq!(encoders.hardware_sessions(), 0);
    assert_eq!(
        Encoders::new(&Ffmpeg, &[Codec::OpenH264], thirty()).hardware_sessions(),
        0
    );
}

#[test]
fn once_videotoolbox_is_set_aside_a_group_opens_openh264() {
    let folder = tempfile::tempdir().unwrap();
    let encoders = Encoders::new(&Ffmpeg, &[Codec::VideoToolbox, Codec::OpenH264], thirty());
    encoders.set_hardware_aside();
    let (_sink, codec) = encoders.open(&folder.path().join("clip.mp4"), WIDTH, HEIGHT).unwrap();
    assert_eq!(codec, Codec::OpenH264);
}

#[test]
fn openh264_writes_a_key_frame_every_second() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("clip.mp4");
    let encoders = Encoders::new(&Ffmpeg, &[Codec::OpenH264], thirty());
    let (mut sink, codec) = encoders.open(&path, WIDTH, HEIGHT).unwrap();
    assert_eq!(codec, Codec::OpenH264);
    sink.open(&path, 70).unwrap();
    let frame = gray();
    for _ in 0..70 {
        sink.push(&frame).unwrap();
    }
    sink.finish().unwrap();
    let keys: Vec<usize> = sequential::read(&path).keys.iter().map(|key| key.get()).collect();
    assert_eq!(keys, [0, 30, 60]);
}

#[test]
fn a_rate_under_one_frame_per_second_keys_every_frame() {
    let slow = FrameRate::new(Rational::new(1, 3).unwrap()).unwrap();
    assert_eq!(Encoders::new(&Ffmpeg, &[], slow).key_interval, NonZeroU32::MIN);
}

#[test]
fn a_clip_whose_folder_is_missing_is_a_destination_error_of_the_clip() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("missing").join("clip.mp4");
    let encoders = Encoders::new(&Ffmpeg, &[Codec::OpenH264], thirty());
    let (mut sink, _) = encoders.open(&path, WIDTH, HEIGHT).unwrap();
    assert!(sink.open(&path, 3).unwrap_err().concerns_the_destination());
}

#[test]
fn a_group_left_without_any_codec_is_refused_without_writing() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("clip.mp4");
    let encoders = Encoders::new(&Ffmpeg, &[Codec::VideoToolbox], thirty());
    encoders.set_hardware_aside();
    let error = encoders.open(&path, WIDTH, HEIGHT).unwrap_err();
    assert!(matches!(error, Error::NoEncoder { path: refused } if refused == path));
    assert_eq!(folder.path().read_dir().unwrap().count(), 0);
}

#[test]
fn an_odd_size_refused_by_h264_names_the_first_clip_of_the_group() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("clip.mp4");
    let encoders = Encoders::new(&Ffmpeg, &[Codec::OpenH264], thirty());
    let error = encoders.open(&path, WIDTH + 1, HEIGHT).unwrap_err();
    assert!(matches!(error, Error::Clip { path: refused, .. } if refused == path));
}

#[test]
fn a_videotoolbox_that_refuses_to_open_hands_its_group_to_openh264_and_stays_allowed() {
    let folder = tempfile::tempdir().unwrap();
    let fake = Fake::refusing(Codec::VideoToolbox);
    let opener = &fake;
    let encoders = Encoders::new(&opener, &[Codec::VideoToolbox, Codec::OpenH264], thirty());
    let (_refused, codec) = encoders
        .open(&folder.path().join("refused.mp4"), WIDTH, HEIGHT)
        .unwrap();
    assert_eq!(codec, Codec::OpenH264);
    assert_eq!(encoders.hardware_sessions(), 1);
}
