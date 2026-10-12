use std::env;
use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::process::Command;

use media::decode::{Packet, Threads};
use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::size::SourceSize;

use super::{Error, Input, Reading, Video, announced, explained};
use crate::lost::Losses;
use crate::testing::{self, FRAMES, HEIGHT, WIDTH};

#[derive(Debug)]
enum Failure {
    Input(Error),
    Visit(FrameIndex),
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Self::Input(error)
    }
}

fn read(path: &Path) -> Result<Reading, Error> {
    Input::open(path, Threads::Automatic)?.read(|_, _| Ok::<(), Error>(()))
}

fn size(width: u32, height: u32) -> SourceSize {
    SourceSize::new(NonZeroU32::new(width).unwrap(), NonZeroU32::new(height).unwrap())
}

#[test]
fn a_clean_video_gives_every_frame_in_order_with_its_pixels_and_passes_the_guard() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), FRAMES);
    let input = Input::open(&path, Threads::Automatic).unwrap();
    let video = input.video().clone();
    let mut seen = Vec::new();
    let reading = input
        .read(|info, frame| {
            let ramp = &frame.y()[..40];
            let mean = f64::from(ramp.iter().map(|pixel| u32::from(*pixel)).sum::<u32>()) / 40.0;
            seen.push((info.index(), info.pts(), frame.width(), frame.height(), mean));
            Ok::<(), Error>(())
        })
        .unwrap();
    assert_eq!(video.path(), path);
    assert_eq!(video.size(), size(WIDTH, HEIGHT));
    assert_eq!(video.frame_rate().get(), Rational::new(30, 1).unwrap());
    assert!(video.time_base().num() > 0);
    assert_eq!(video.announced_frames().get(), usize::from(FRAMES));
    assert_eq!(reading.video(), &video);
    assert_eq!(reading.frames().len(), usize::from(FRAMES));
    assert_eq!(video.start(), Some(Pts::new(0)));
    assert_eq!(reading.losses(), Losses::default());
    assert_eq!(seen.len(), usize::from(FRAMES));
    for (position, (index, _, width, height, _)) in seen.iter().enumerate() {
        assert_eq!((*index, *width, *height), (FrameIndex::new(position), WIDTH, HEIGHT));
    }
    for (before, after) in seen.iter().zip(seen.iter().skip(1)) {
        assert!(after.1 > before.1);
        assert!((after.4 - before.4 - 3.0).abs() < 1.0, "{} then {}", before.4, after.4);
    }
}

#[test]
fn frames_lost_at_a_timestamp_jump_are_expected_by_the_guard() {
    let folder = tempfile::tempdir().unwrap();
    let reading = read(&testing::damaged(folder.path(), 2, 3)).unwrap();
    let decoded = reading.frames().len();
    assert!(decoded < usize::from(FRAMES));
    assert_eq!(
        reading.losses(),
        Losses {
            at_jumps: usize::from(FRAMES) - decoded,
            ..Losses::default()
        }
    );
}

#[test]
fn frames_unreadable_after_the_last_one_are_a_loss_at_the_edge_not_a_refusal() {
    let folder = tempfile::tempdir().unwrap();
    let reading = read(&testing::damaged(folder.path(), 3, 4)).unwrap();
    let decoded = reading.frames().len();
    assert!(decoded < usize::from(FRAMES));
    assert_eq!(
        reading.losses(),
        Losses {
            after_last: usize::from(FRAMES) - decoded,
            ..Losses::default()
        }
    );
}

#[test]
fn frames_lost_before_the_first_one_are_a_loss_at_the_edge_found_from_its_timestamp() {
    let path = reference::fixture::fixture("two_gops.mp4").unwrap();
    let input = Input::open(&path, Threads::Automatic)
        .unwrap()
        .with_seam(Box::new(|packet: &mut Packet<'_>| {
            if packet.ordinal() == 0 {
                let half = packet.size() / 2;
                packet.shrink(half);
            }
        }));
    let reading = input.read(|_, _| Ok::<(), Error>(())).unwrap();
    assert_eq!(reading.frames().len(), 10);
    assert!(reading.frames()[0].is_key());
    assert_eq!(
        reading.losses(),
        Losses {
            before_first: 10,
            ..Losses::default()
        }
    );
}

#[test]
fn a_loss_that_neither_a_jump_nor_an_edge_explains_fails_the_video_and_names_it() {
    let video = Video {
        path: PathBuf::from("night.mp4"),
        size: size(1440, 1080),
        frame_rate: FrameRate::new(Rational::new(30, 1).unwrap()).unwrap(),
        time_base: Rational::new(1, 15_360).unwrap(),
        start: Some(Pts::new(0)),
        announced_frames: NonZeroUsize::new(30).unwrap(),
    };
    let pts: Vec<_> = (0..20).map(|frame| Some(Pts::new(frame * 512))).collect();
    let error = explained(&video, &pts, 3).unwrap_err();
    let Error::FrameCount {
        path,
        announced,
        losses,
        decoded,
    } = &error
    else {
        panic!("{error:?}")
    };
    assert_eq!((path.as_path(), *announced, *decoded), (Path::new("night.mp4"), 30, 20));
    assert_eq!(losses.total(), 3);
    assert!(error.to_string().contains("night.mp4"), "{error}");
    assert_eq!(explained(&video, &pts, 10).unwrap().after_last, 10);
}

#[test]
fn a_video_that_announces_no_frame_is_refused() {
    let path = Path::new("empty.mp4");
    assert!(matches!(
        announced(path, None),
        Err(Error::NoAnnouncedFrames { path: named }) if named == path
    ));
    assert_eq!(announced(path, NonZeroU64::new(9011)).unwrap().get(), 9011);
}

#[test]
fn the_engine_entry_takes_a_video_of_any_size() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), 1);

    let input = Input::open(&path, Threads::Automatic).unwrap();

    assert_eq!(input.video().size(), size(WIDTH, HEIGHT));
}

#[test]
fn a_missing_video_is_a_decode_error_that_names_it() {
    let path = PathBuf::from("absent.mp4");
    let refused = Input::open(&path, Threads::Automatic).unwrap_err();
    assert!(matches!(&refused, Error::Decode { path: named, .. } if *named == path));
    assert!(refused.to_string().contains("absent.mp4"));
}

#[test]
fn every_refusal_names_the_video() {
    let path = PathBuf::from("night.mp4");
    let refusals = [
        Error::NoAnnouncedFrames { path: path.clone() },
        Error::Frame {
            path: path.clone(),
            source: media::frame::Error::EmptySize { width: 0, height: 0 },
        },
    ];
    for refusal in refusals {
        assert!(refusal.to_string().contains("night.mp4"), "{refusal}");
    }
}

#[test]
fn the_first_visitor_error_stops_reading_and_is_returned_as_is() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), FRAMES);
    let input = Input::open(&path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    let mut visited = 0;
    let outcome = input.read(|info, _| {
        visited += 1;
        if info.index() == FrameIndex::new(3) {
            Err(Failure::Visit(info.index()))
        } else {
            Ok(())
        }
    });
    assert!(matches!(outcome, Err(Failure::Visit(index)) if index == FrameIndex::new(3)));
    assert_eq!(visited, 4);
}

#[test]
fn a_decoder_failure_mid_video_reaches_the_caller_through_its_own_error_type() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), FRAMES);
    let input = Input::open(&path, Threads::Automatic)
        .unwrap()
        .with_seam(Box::new(|packet: &mut Packet<'_>| {
            if packet.ordinal() == 5 {
                packet.shrink(0);
            }
        }));
    let outcome = input.read(|_, _| Ok::<(), Failure>(()));
    assert!(
        matches!(&outcome, Err(Failure::Input(Error::Decode { path: named, .. })) if *named == path),
        "{outcome:?}"
    );
}

#[test]
#[ignore = "run by reading_a_damaged_video_writes_no_decoder_message"]
fn read_a_damaged_video() {
    let folder = tempfile::tempdir().unwrap();
    let reading = read(&testing::damaged(folder.path(), 2, 3)).unwrap();
    let unreadable: u32 = reading.frames().iter().map(|info| info.unreadable_packets()).sum();
    assert!(unreadable > 0);
}

#[test]
fn reading_a_damaged_video_writes_no_decoder_message() {
    let output = Command::new(env::current_exe().unwrap())
        .args(["--exact", "input::tests::read_a_damaged_video", "--include-ignored"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    assert!(!stderr.contains(" @ 0x"), "{stderr}");
}
