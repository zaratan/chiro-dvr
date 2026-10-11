use std::fs;
use std::path::Path;

use ffmpeg::format::Pixel;
use ffmpeg::util::color::Range;
use ffmpeg_next as ffmpeg;
use types::rational::Rational;

use super::{Decoder, Error, color_range, frame_rate, same_size, unpack};
use crate::encode::Codec;
use crate::frame::{ColorRange, YuvFrame};
use crate::testing::{self, HEIGHT, WIDTH};

fn frames(decoder: &mut Decoder) -> (u32, u32) {
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let (mut count, mut unreadable) = (0, 0);
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        count += 1;
        unreadable += info.unreadable_packets();
    }
    (count, unreadable + decoder.unreadable_packets_after_last_frame())
}

fn damaged(folder: &Path, from_quarter: usize, to_quarter: usize) -> std::path::PathBuf {
    let path = testing::encode(folder, Codec::OpenH264, 30);
    let mut bytes = fs::read(&path).unwrap();
    let payload = bytes.windows(4).position(|window| window == b"mdat").unwrap() + 4;
    let size: [u8; 4] = bytes[payload - 8..payload - 4].try_into().unwrap();
    let quarter = (usize::try_from(u32::from_be_bytes(size)).unwrap() - 8) / 4;
    bytes[payload + from_quarter * quarter..payload + to_quarter * quarter].fill(0xff);
    fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn a_missing_file_is_an_open_error_not_a_panic() {
    assert!(matches!(
        Decoder::open(Path::new("absent.mp4")),
        Err(Error::Open { .. })
    ));
}

#[test]
fn a_file_without_video_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let text = folder.path().join("notes.mp4");
    fs::write(&text, "not a video").unwrap();
    assert!(matches!(Decoder::open(&text), Err(Error::Open { .. })));
}

#[test]
fn a_path_ffmpeg_cannot_take_is_refused_without_panic() {
    assert!(matches!(
        Decoder::open(Path::new("cl\0ip.mp4")),
        Err(Error::UnusablePath { .. })
    ));
}

#[test]
fn the_stream_describes_what_was_encoded() {
    let folder = tempfile::tempdir().unwrap();
    let decoder = Decoder::open(&testing::encode(folder.path(), Codec::OpenH264, 3)).unwrap();
    let stream = decoder.stream();
    assert_eq!((stream.width(), stream.height()), (WIDTH, HEIGHT));
    assert_eq!(stream.frame_rate(), Rational::new(30, 1).unwrap());
    assert!(stream.time_base().num() > 0);
    assert_eq!(stream.range(), ColorRange::Unspecified);
}

#[test]
fn a_debugged_decoder_shows_its_stream_not_ffmpeg_internals() {
    let folder = tempfile::tempdir().unwrap();
    let decoder = Decoder::open(&testing::encode(folder.path(), Codec::OpenH264, 1)).unwrap();
    assert!(format!("{decoder:?}").contains("info: StreamInfo"));
}

#[test]
fn the_first_frame_is_a_key_frame_with_the_encoded_pixels_and_openh264_leaves_limited_range_implicit() {
    let folder = tempfile::tempdir().unwrap();
    let mut decoder = Decoder::open(&testing::encode(folder.path(), Codec::OpenH264, 3)).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let info = decoder.next(&mut frame).unwrap().unwrap();
    let expected = testing::pattern(0);
    let error = frame
        .y()
        .iter()
        .zip(expected.y())
        .map(|(a, b)| u32::from(a.abs_diff(*b)))
        .max()
        .unwrap();
    assert!(info.key());
    assert_eq!(
        (frame.width(), frame.height(), frame.range()),
        (WIDTH, HEIGHT, ColorRange::Unspecified)
    );
    assert!(error < 24, "luma differs by {error}");
}

#[test]
fn next_keeps_returning_none_after_the_end() {
    let folder = tempfile::tempdir().unwrap();
    let mut decoder = Decoder::open(&testing::encode(folder.path(), Codec::OpenH264, 2)).unwrap();
    assert_eq!(frames(&mut decoder), (2, 0));
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    assert!(decoder.next(&mut frame).unwrap().is_none());
    assert!(decoder.next(&mut frame).unwrap().is_none());
}

#[test]
fn a_damaged_payload_is_counted_and_decoding_goes_on() {
    let folder = tempfile::tempdir().unwrap();
    let (count, unreadable) = frames(&mut Decoder::open(&damaged(folder.path(), 2, 3)).unwrap());
    assert!(
        count > 0 && unreadable > 0,
        "{count} frames, {unreadable} unreadable packets"
    );
}

#[test]
fn damage_after_the_last_frame_is_still_counted() {
    let folder = tempfile::tempdir().unwrap();
    let mut decoder = Decoder::open(&damaged(folder.path(), 3, 4)).unwrap();
    let (count, _) = frames(&mut decoder);
    assert!(count > 0);
    assert!(decoder.unreadable_packets_after_last_frame() > 0);
}

#[test]
fn the_average_rate_wins_and_the_real_rate_replaces_an_unknown_one() {
    let average = ffmpeg::Rational::new(90_110_000, 3_000_927);
    let real = ffmpeg::Rational::new(10_000, 333);
    let unknown = ffmpeg::Rational::new(0, 0);
    assert_eq!(
        frame_rate(average, real).unwrap(),
        Rational::new(90_110_000, 3_000_927).unwrap()
    );
    assert_eq!(frame_rate(unknown, real).unwrap(), Rational::new(10_000, 333).unwrap());
    assert!(matches!(frame_rate(unknown, unknown), Err(Error::UnknownFrameRate)));
}

#[test]
fn tv_range_yuv_like_the_thermal_videos_is_read_as_limited_range() {
    assert_eq!(color_range(Pixel::YUV420P, Range::MPEG).unwrap(), ColorRange::Limited);
}

#[test]
fn full_range_jpeg_yuv_is_read_as_full_range() {
    assert_eq!(
        color_range(Pixel::YUVJ420P, Range::Unspecified).unwrap(),
        ColorRange::Full
    );
    assert_eq!(color_range(Pixel::YUV420P, Range::JPEG).unwrap(), ColorRange::Full);
    assert_eq!(
        color_range(Pixel::YUV420P, Range::Unspecified).unwrap(),
        ColorRange::Unspecified
    );
}

#[test]
fn other_pixel_formats_are_refused() {
    assert!(matches!(
        color_range(Pixel::YUV420P10LE, Range::MPEG),
        Err(Error::UnsupportedPixelFormat { format: "yuv420p10le" })
    ));
}

#[test]
fn decoder_encoder_and_frame_can_move_to_another_thread() {
    fn movable<T: Send>() {}
    movable::<Decoder>();
    movable::<crate::encode::Encoder>();
    movable::<YuvFrame>();
}

#[test]
fn a_frame_whose_size_differs_from_the_stream_is_refused() {
    assert!(same_size((1440, 1080), (1440, 1080)).is_ok());
    assert!(matches!(
        same_size((1440, 1080), (720, 540)),
        Err(Error::SizeChanged {
            found_width: 720,
            found_height: 540,
            ..
        })
    ));
}

#[test]
fn a_decoded_plane_shorter_than_its_size_is_refused() {
    let mut frame = YuvFrame::new(4, 2, ColorRange::Limited).unwrap();
    let luma = [1_u8; 16];
    let chroma = [2_u8; 2];
    assert!(unpack([(&luma, 8), (&chroma, 2), (&chroma, 2)], &mut frame).is_ok());
    assert_eq!(frame.y(), [1; 8]);
    assert!(matches!(
        unpack([(&luma[..5], 8), (&chroma, 2), (&chroma, 2)], &mut frame),
        Err(Error::TruncatedFrame)
    ));
}
