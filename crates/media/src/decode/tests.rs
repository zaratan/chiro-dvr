use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::Path;

use ffmpeg::format::Pixel;
use ffmpeg::util::color::Range;
use ffmpeg_next as ffmpeg;
use types::frame::{FrameIndex, Pts};
use types::rational::Rational;

use super::{
    ActiveThreads, Decoder, Error, FrameInfo, Packet, Threads, color_range, frame_rate, known_pts, same_size, unpack,
    upright,
};
use crate::decode_errors::DecodeErrors;
use crate::encode::Codec;
use crate::frame::{ColorRange, YuvFrame};
use crate::testing::{self, HEIGHT, WIDTH};

fn open(path: &Path) -> Decoder {
    Decoder::open(path, Threads::Automatic).unwrap()
}

fn infos(mut decoder: Decoder) -> Vec<FrameInfo> {
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut infos = Vec::new();
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        infos.push(info);
    }
    infos
}

fn frames(decoder: &mut Decoder) -> (u32, u32) {
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let (mut count, mut unreadable) = (0, 0);
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        count += 1;
        unreadable += info.unreadable_packets();
    }
    (count, unreadable + decoder.unreadable_packets_after_last_frame())
}

fn corrupted_early_in_payload(folder: &Path) -> std::path::PathBuf {
    let path = testing::encode(folder, Codec::OpenH264, 30);
    let mut bytes = fs::read(&path).unwrap();
    let payload = bytes.windows(4).position(|window| window == b"mdat").unwrap() + 4;
    let early = payload + (bytes.len() - payload) / 10;
    bytes[early..early + 16].fill(0xff);
    fs::write(&path, bytes).unwrap();
    path
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
        Decoder::open(Path::new("absent.mp4"), Threads::Automatic),
        Err(Error::Open { .. })
    ));
}

#[test]
fn a_file_without_video_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let text = folder.path().join("notes.mp4");
    fs::write(&text, "not a video").unwrap();
    assert!(matches!(
        Decoder::open(&text, Threads::Automatic),
        Err(Error::Open { .. })
    ));
}

#[test]
fn a_path_ffmpeg_cannot_take_is_refused_without_panic() {
    assert!(matches!(
        Decoder::open(Path::new("cl\0ip.mp4"), Threads::Automatic),
        Err(Error::UnusablePath { .. })
    ));
}

#[test]
fn the_stream_describes_what_was_encoded() {
    let folder = tempfile::tempdir().unwrap();
    let decoder = open(&testing::encode(folder.path(), Codec::OpenH264, 3));
    let stream = decoder.stream();
    assert_eq!((stream.width(), stream.height()), (WIDTH, HEIGHT));
    assert_eq!(
        (stream.size().width().get(), stream.size().height().get()),
        (WIDTH, HEIGHT)
    );
    assert_eq!(stream.frame_rate().get(), Rational::new(30, 1).unwrap());
    assert!(stream.time_base().num() > 0);
    assert_eq!(stream.range(), ColorRange::Limited);
    assert_eq!(stream.announced_frames(), NonZeroU64::new(3));
    assert_eq!(stream.start(), Some(Pts::new(0)));
}

#[test]
fn a_debugged_decoder_shows_its_stream_not_ffmpeg_internals() {
    let folder = tempfile::tempdir().unwrap();
    let decoder = open(&testing::encode(folder.path(), Codec::OpenH264, 1));
    assert!(format!("{decoder:?}").contains("info: StreamInfo"));
}

#[test]
fn the_first_frame_is_a_key_frame_with_the_encoded_pixels_in_the_written_limited_range() {
    let folder = tempfile::tempdir().unwrap();
    let mut decoder = open(&testing::encode(folder.path(), Codec::OpenH264, 3));
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
    assert!(info.is_key());
    assert_eq!(
        (frame.width(), frame.height(), frame.range()),
        (WIDTH, HEIGHT, ColorRange::Limited)
    );
    assert!(error < 24, "luma differs by {error}");
}

#[test]
fn next_keeps_returning_none_after_the_end() {
    let folder = tempfile::tempdir().unwrap();
    let mut decoder = open(&testing::encode(folder.path(), Codec::OpenH264, 2));
    assert_eq!(frames(&mut decoder), (2, 0));
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    assert!(decoder.next(&mut frame).unwrap().is_none());
    assert!(decoder.next(&mut frame).unwrap().is_none());
}

#[test]
fn a_damaged_payload_is_counted_and_decoding_goes_on() {
    let folder = tempfile::tempdir().unwrap();
    let (count, unreadable) = frames(&mut open(&damaged(folder.path(), 2, 3)));
    assert!(
        count > 0 && unreadable > 0,
        "{count} frames, {unreadable} unreadable packets"
    );
}

#[test]
fn damage_after_the_last_frame_is_still_counted() {
    let folder = tempfile::tempdir().unwrap();
    let mut decoder = open(&damaged(folder.path(), 3, 4));
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
        frame_rate(average, real).unwrap().get(),
        Rational::new(90_110_000, 3_000_927).unwrap()
    );
    assert_eq!(
        frame_rate(unknown, real).unwrap().get(),
        Rational::new(10_000, 333).unwrap()
    );
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

#[test]
fn frames_are_numbered_in_decoding_order_with_one_timestamp_step_apart() {
    let folder = tempfile::tempdir().unwrap();
    let infos = infos(open(&testing::encode(folder.path(), Codec::OpenH264, 10)));
    let step = infos[1].pts().unwrap().get();
    assert!(step > 0);
    for (position, info) in infos.iter().enumerate() {
        assert_eq!(info.index(), FrameIndex::new(position));
        assert_eq!(info.pts(), Some(Pts::new(step * i64::try_from(position).unwrap())));
        assert!(info.decode_errors().is_empty());
    }
}

#[test]
fn a_corrupted_slice_is_concealed_and_reports_its_raw_error_flags_untruncated() {
    let folder = tempfile::tempdir().unwrap();
    let infos = infos(open(&corrupted_early_in_payload(folder.path())));
    let flagged: Vec<u32> = infos
        .iter()
        .map(|info| info.decode_errors())
        .filter(|errors| !errors.is_empty())
        .map(DecodeErrors::bits)
        .collect();
    assert_eq!(infos.len(), 30);
    assert_eq!(flagged, [12]);
}

fn fingerprints(mut decoder: Decoder) -> Vec<(FrameInfo, u64)> {
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut fingerprints = Vec::new();
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        let mut hasher = DefaultHasher::new();
        (frame.y(), frame.u(), frame.v()).hash(&mut hasher);
        fingerprints.push((info, hasher.finish()));
    }
    fingerprints
}

#[test]
fn one_decoding_thread_gives_the_same_frames_and_pixels_as_automatic_threading() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 30);
    let single = Decoder::open(&path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    assert_eq!(fingerprints(single), fingerprints(open(&path)));
}

#[test]
fn the_frame_table_of_a_concealed_slice_does_not_depend_on_threading() {
    let folder = tempfile::tempdir().unwrap();
    let path = corrupted_early_in_payload(folder.path());
    let single = Decoder::open(&path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    assert_eq!(infos(single), infos(open(&path)));
}

#[test]
fn the_seam_sees_every_video_packet_once_in_reading_order() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 12);
    let (sender, receiver) = std::sync::mpsc::channel();
    let decoder = open(&path).with_seam(Box::new(move |packet: &mut Packet<'_>| {
        sender.send((packet.ordinal(), packet.is_key(), packet.size())).unwrap();
    }));
    assert_eq!(infos(decoder).len(), 12);
    let seen: Vec<_> = receiver.iter().collect();
    assert_eq!(
        seen.iter().map(|(ordinal, ..)| *ordinal).collect::<Vec<_>>(),
        (0..12).collect::<Vec<_>>()
    );
    assert!(seen[0].1);
    assert!(seen.iter().all(|(.., size)| *size > 0));
}

#[test]
fn a_packet_cut_by_the_seam_reaches_the_decoder_as_unreadable() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 12);
    let decoder = open(&path).with_seam(Box::new(|packet: &mut Packet<'_>| {
        if packet.ordinal() == 5 {
            let half = packet.size() / 2;
            packet.shrink(half);
        }
    }));
    let infos = infos(decoder);
    assert_eq!(infos.len(), 11);
    assert_eq!(infos.iter().map(|info| info.unreadable_packets()).sum::<u32>(), 1);
}

#[test]
fn a_debugged_packet_shows_its_rank_key_and_size() {
    let folder = tempfile::tempdir().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let decoder = open(&testing::encode(folder.path(), Codec::OpenH264, 1)).with_seam(Box::new(
        move |packet: &mut Packet<'_>| sender.send(format!("{packet:?}")).unwrap(),
    ));
    assert_eq!(infos(decoder).len(), 1);
    let shown = receiver.recv().unwrap();
    assert!(shown.starts_with("Packet { ordinal: 0, key: true, size: "), "{shown}");
}

#[test]
fn an_unknown_start_is_none_rather_than_a_sentinel_timestamp() {
    assert_eq!(known_pts(ffmpeg::ffi::AV_NOPTS_VALUE), None);
    assert_eq!(known_pts(0), Some(Pts::new(0)));
}

#[test]
fn a_frame_stored_bottom_up_is_refused_before_its_planes_are_read() {
    assert!(upright([1440, 720, 720]).is_ok());
    let negative = (-1440_isize).cast_unsigned();
    assert!(matches!(upright([negative, 720, 720]), Err(Error::FlippedFrame)));
    assert!(matches!(upright([1440, 720, negative]), Err(Error::FlippedFrame)));
}

#[test]
fn one_requested_thread_runs_alone_and_more_run_as_frame_threads() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 2);
    let single = Decoder::open(&path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    assert_eq!(single.active_threads(), ActiveThreads::Single);
    let four = NonZeroUsize::new(4).unwrap();
    let several = Decoder::open(&path, Threads::Exactly(four)).unwrap();
    assert_eq!(several.active_threads(), ActiveThreads::Frame(four));
}

#[test]
fn automatic_threading_decodes_with_several_frame_threads_on_a_multicore_machine() {
    let folder = tempfile::tempdir().unwrap();
    let decoder = open(&testing::encode(folder.path(), Codec::OpenH264, 2));
    let cores = std::thread::available_parallelism().unwrap().get();
    let active = decoder.active_threads();
    assert!(
        cores == 1 || matches!(active, ActiveThreads::Frame(count) if count.get() > 1),
        "{cores} cores, {active:?}"
    );
}

#[test]
fn bytes_written_by_the_seam_reach_the_decoder() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), Codec::OpenH264, 12);
    let decoder = open(&path).with_seam(Box::new(|packet: &mut Packet<'_>| {
        if packet.ordinal() == 5 {
            packet.data_mut().unwrap().fill(0xff);
        }
    }));
    let infos = infos(decoder);
    assert_eq!(infos.len(), 11);
    assert_eq!(infos.iter().map(|info| info.unreadable_packets()).sum::<u32>(), 1);
}
