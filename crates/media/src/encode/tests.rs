use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use ffmpeg::util::color::{Primaries, Range, Space, TransferCharacteristic};
use ffmpeg_next as ffmpeg;
use types::rational::Rational;

use super::{Codec, Encoder, Error, Settings};
use crate::decode::{Decoder, Threads};
use crate::frame::{ColorRange, YuvFrame};
use crate::testing::{self, HEIGHT, WIDTH};

fn thirty() -> Rational {
    Rational::new(30, 1).unwrap()
}

fn decoded_frames(path: &Path) -> (u32, u32, u32) {
    let mut decoder = Decoder::open(path, Threads::Automatic).unwrap();
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

fn key_frames(settings: &Settings, folder: &Path) -> Vec<usize> {
    let path = folder.join("clip.mp4");
    let mut encoder = Encoder::create(&path, settings).unwrap();
    for index in 0..25 {
        encoder.push(&testing::pattern(index)).unwrap();
    }
    encoder.finish().unwrap();
    let mut decoder = Decoder::open(&path, Threads::Automatic).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut keys = Vec::new();
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        if info.is_key() {
            keys.push(info.index().get());
        }
    }
    keys
}

#[test]
fn openh264_writes_a_single_key_frame_unless_given_an_interval() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    assert_eq!(key_frames(&settings, folder.path()), [0]);
    let every_ten = settings.with_key_interval(NonZeroU32::new(10).unwrap());
    assert_eq!(key_frames(&every_ten, folder.path()), [0, 10, 20]);
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
    let path = Path::new("cl\0ip.mp4");
    assert!(matches!(
        Encoder::create(path, &settings),
        Err(Error::UnusablePath { .. })
    ));
}

#[cfg(target_os = "macos")]
#[test]
fn videotoolbox_hardware_at_q65_encodes_one_second() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::VideoToolbox, 30);
    assert_eq!(decoded_frames(&path), (30, WIDTH, HEIGHT));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn videotoolbox_is_missing_outside_macos() {
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::VideoToolbox).unwrap();
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

fn colors_written_in_the_h264_stream(path: &Path) -> (Range, Space, Primaries, TransferCharacteristic) {
    let mut input = ffmpeg::format::input(path).unwrap();
    let stream = input.streams().best(ffmpeg::media::Type::Video).unwrap();
    let index = stream.index();
    let mut context = ffmpeg::codec::context::Context::from_parameters(stream.parameters()).unwrap();
    // SAFETY: `context` owns its AVCodecContext and is not shared.
    let pointer = unsafe { context.as_mut_ptr() };
    // SAFETY: `pointer` is the live, unopened context above, borrowed only for the next four writes.
    let raw = unsafe { &mut *pointer };
    raw.color_range = Range::Unspecified.into();
    raw.colorspace = Space::Unspecified.into();
    raw.color_primaries = Primaries::Unspecified.into();
    raw.color_trc = TransferCharacteristic::Unspecified.into();
    let mut decoder = context.decoder().video().unwrap();
    let mut frame = ffmpeg::frame::Video::empty();
    for (stream, packet) in input.packets() {
        if stream.index() == index {
            decoder.send_packet(&packet).unwrap();
            if decoder.receive_frame(&mut frame).is_ok() {
                break;
            }
        }
    }
    if frame.width() == 0 {
        decoder.send_eof().unwrap();
        decoder.receive_frame(&mut frame).unwrap();
    }
    (
        frame.color_range(),
        frame.color_space(),
        frame.color_primaries(),
        frame.color_transfer_characteristic(),
    )
}

fn encoded_at(folder: &Path, frame_rate: Rational) -> PathBuf {
    let path = folder.join("rate.mp4");
    let settings = Settings::new(WIDTH, HEIGHT, frame_rate, Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&path, &settings).unwrap();
    for index in 0..30 {
        encoder.push(&testing::pattern(index)).unwrap();
    }
    encoder.finish().unwrap();
    path
}

fn is_send<T: Send>() {}

#[test]
fn openh264_writes_limited_range_bt709_in_the_h264_stream_not_only_in_the_container() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 3);
    assert_eq!(
        colors_written_in_the_h264_stream(&path),
        (
            Range::MPEG,
            Space::BT709,
            Primaries::BT709,
            TransferCharacteristic::BT709
        )
    );
}

#[test]
fn openh264_encodes_in_high_profile_rather_than_its_constrained_baseline_default() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 3);
    let input = ffmpeg::format::input(&path).unwrap();
    let stream = input.streams().best(ffmpeg::media::Type::Video).unwrap();
    let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters()).unwrap();
    let decoder = context.decoder().video().unwrap();
    assert_eq!(
        decoder.profile(),
        ffmpeg::codec::Profile::H264(ffmpeg::codec::profile::H264::High)
    );
}

#[test]
fn a_non_integer_rate_reads_back_exactly() {
    let folder = tempfile::tempdir().unwrap();
    let rate = Rational::new(10_000, 333).unwrap();
    let path = encoded_at(folder.path(), rate);
    assert_eq!(
        Decoder::open(&path, Threads::Automatic)
            .unwrap()
            .stream()
            .frame_rate()
            .get(),
        rate
    );
}

#[test]
fn a_rate_four_times_slower_reads_back_exactly() {
    let folder = tempfile::tempdir().unwrap();
    let rate = Rational::new(10_000, 333 * 4).unwrap();
    let path = encoded_at(folder.path(), rate);
    assert_eq!(
        Decoder::open(&path, Threads::Automatic)
            .unwrap()
            .stream()
            .frame_rate()
            .get(),
        rate
    );
}

#[test]
fn the_destination_does_not_exist_until_finish() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("clip.mp4");
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&path, &settings).unwrap();
    encoder.push(&testing::pattern(0)).unwrap();
    assert!(!path.exists());
    encoder.finish().unwrap();
    assert!(path.exists());
}

#[test]
fn an_encoder_can_move_to_another_thread() {
    is_send::<Encoder>();
}

#[test]
fn an_encoder_says_which_codec_writes_it() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let encoder = Encoder::create(&folder.path().join("clip.mp4"), &settings).unwrap();
    assert_eq!(encoder.codec(), Codec::OpenH264);
}

#[test]
fn each_codec_has_a_name_for_the_fallback_report() {
    assert_eq!(Codec::OpenH264.to_string(), "openh264");
    assert_eq!(Codec::VideoToolbox.to_string(), "VideoToolbox hardware");
}

#[test]
fn a_videotoolbox_session_counts_more_memory_than_openh264_for_its_outside_process_service() {
    assert!(Codec::OpenH264.footprint() < Codec::VideoToolbox.footprint());
}

#[cfg(target_os = "macos")]
#[test]
fn videotoolbox_writes_limited_range_bt709_in_the_h264_stream() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::VideoToolbox, 3);
    assert_eq!(
        colors_written_in_the_h264_stream(&path),
        (
            Range::MPEG,
            Space::BT709,
            Primaries::BT709,
            TransferCharacteristic::BT709
        )
    );
}

#[test]
fn the_unfinished_clip_is_written_under_a_dot_name_beside_it() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&folder.path().join("clip.mp4"), &settings).unwrap();
    encoder.push(&testing::pattern(0)).unwrap();
    assert!(folder.path().join(".clip.mp4.partial").exists());
    encoder.finish().unwrap();
    assert!(!folder.path().join(".clip.mp4.partial").exists());
}

#[test]
fn a_partial_file_left_by_an_earlier_run_is_overwritten() {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join(".clip.mp4.partial"), "left over").unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&folder.path().join("clip.mp4"), &settings).unwrap();
    encoder.push(&testing::pattern(0)).unwrap();
    encoder.finish().unwrap();
    assert_eq!(decoded_frames(&folder.path().join("clip.mp4")).0, 1);
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn a_clip_gets_the_same_permissions_as_any_file_the_user_creates() {
    use std::os::unix::fs::PermissionsExt;
    let folder = tempfile::tempdir().unwrap();
    let ordinary = folder.path().join("ordinary");
    fs::File::create(&ordinary).unwrap();
    let path = folder.path().join("clip.mp4");
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&path, &settings).unwrap();
    encoder.push(&testing::pattern(0)).unwrap();
    encoder.finish().unwrap();
    let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode();
    assert_eq!(mode(&path), mode(&ordinary));
}

#[test]
fn a_destination_without_a_file_name_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(WIDTH, HEIGHT, thirty(), Codec::OpenH264).unwrap();
    assert!(matches!(
        Encoder::create(&folder.path().join(".."), &settings),
        Err(Error::UnusablePath { .. })
    ));
}

#[test]
fn only_errors_about_the_destination_concern_it() {
    let io = || std::io::Error::other("refused");
    let path = PathBuf::from("clip.mp4");
    assert!(Error::UnusablePath { path: path.clone() }.concerns_the_destination());
    assert!(
        Error::Create {
            path: path.clone(),
            source: io()
        }
        .concerns_the_destination()
    );
    let library = || crate::failure::LibraryError::new(ffmpeg::Error::InvalidData);
    assert!(
        Error::OpenFile {
            path: path.clone(),
            source: library()
        }
        .concerns_the_destination()
    );
    assert!(
        !Error::Writing {
            path: path.clone(),
            source: library()
        }
        .concerns_the_destination()
    );
    assert!(!Error::NoKeyFrame { path: path.clone() }.concerns_the_destination());
    assert!(Error::Persist { path, source: io() }.concerns_the_destination());
    assert!(!Error::FrameLayout.concerns_the_destination());
    assert!(!Error::FullRange.concerns_the_destination());
}
