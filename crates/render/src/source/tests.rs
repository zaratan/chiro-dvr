use std::path::{Path, PathBuf};
use std::thread;

use media::decode::Threads;
use types::frame::{FrameIndex, Pts};
use types::span::Span;
use types::timeline::Timeline;

use super::{Error, Origin, Restart, Source};
use crate::sequential::{self, Sequential};
use crate::testing;

#[derive(Debug)]
enum Stop {
    Source(Error),
    Visitor,
}

impl From<Error> for Stop {
    fn from(error: Error) -> Self {
        Self::Source(error)
    }
}

fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

fn losing_frames(folder: &Path) -> (PathBuf, Sequential) {
    let path = testing::losing_frames(folder, 5..6, 12);
    let read = sequential::read(&path);
    (path, read)
}

fn serve(source: &Source, span: Span) -> Result<(Origin, Vec<(FrameIndex, u64)>), Error> {
    let mut served = Vec::new();
    let origin = source.serve(span, |frame, pixels| {
        served.push((frame, sequential::fingerprint(pixels)));
        Ok::<(), Error>(())
    })?;
    Ok((origin, served))
}

fn forged(read: &Sequential, pts: Vec<Option<Pts>>) -> Timeline {
    Timeline::new(pts, read.timeline.time_base(), read.timeline.rate()).unwrap()
}

fn all_pts(read: &Sequential) -> Vec<Option<Pts>> {
    (0..read.timeline.frame_count())
        .map(|frame| read.timeline.pts(FrameIndex::new(frame)))
        .collect()
}

#[test]
fn every_window_of_a_video_that_loses_frames_serves_its_frames_once_in_order_with_sequential_pixels() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let frames = read.timeline.frame_count();
    assert!(frames < testing::FRAMES);
    assert!(read.keys.iter().any(|key| read.errors.contains(key)));
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    let mut sought = 0;
    let windows = (0..frames).map(|first| span(first, (first + 3).min(frames - 1)));
    for window in windows.chain([span(0, frames - 1)]) {
        let (origin, served) = serve(&source, window).unwrap();
        let expected: Vec<(FrameIndex, u64)> = (window.first().get()..=window.last().get())
            .map(|frame| (FrameIndex::new(frame), read.fingerprints[frame]))
            .collect();
        assert_eq!(served, expected, "{window:?}");
        let Origin::Key(key) = origin else {
            panic!("{window:?} was read from the start");
        };
        assert!(key <= window.first() && read.keys.contains(&key) && !read.errors.contains(&key));
        sought += usize::from(key.get() > 0);
    }
    assert!(sought > frames / 2);
}

#[test]
fn without_a_clean_key_frame_the_window_is_read_from_the_start() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let source = Source::new(&path, &read.timeline, &[], &read.errors).unwrap();
    let (origin, served) = serve(&source, span(20, 23)).unwrap();
    assert!(matches!(origin, Origin::Start(Restart::NoCleanKey)));
    let pixels: Vec<u64> = served.into_iter().map(|(_, pixels)| pixels).collect();
    assert_eq!(pixels, read.fingerprints[20..=23]);
}

#[test]
fn a_key_frame_the_container_cannot_land_on_sends_the_reading_back_to_the_start() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let first = read.timeline.pts(FrameIndex::new(0)).unwrap().get();
    let mut pts = vec![Some(Pts::new(first - 1000))];
    pts.extend(all_pts(&read));
    let timeline = forged(&read, pts);
    let source = Source::new(&path, &timeline, &[FrameIndex::new(0)], &[]).unwrap();
    let (origin, served) = serve(&source, span(5, 8)).unwrap();
    assert!(matches!(
        origin,
        Origin::Start(Restart::Overshot { landing, first }) if landing == FrameIndex::new(0) && first == FrameIndex::new(1)
    ));
    let pixels: Vec<u64> = served.into_iter().map(|(_, pixels)| pixels).collect();
    assert_eq!(pixels, read.fingerprints[4..=7]);
}

#[test]
fn a_window_past_the_last_frame_is_refused_before_any_decoding() {
    let folder = tempfile::tempdir().unwrap();
    let (_, read) = losing_frames(folder.path());
    let frames = read.timeline.frame_count();
    let source = Source::new(Path::new("absent.mp4"), &read.timeline, &read.keys, &[]).unwrap();
    assert!(matches!(
        serve(&source, span(frames - 2, frames)),
        Err(Error::OutsideVideo { frames: count, .. }) if count == frames
    ));
}

#[test]
fn a_window_over_a_frame_without_pts_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let mut pts = all_pts(&read);
    pts[6] = None;
    let timeline = forged(&read, pts);
    let source = Source::new(&path, &timeline, &read.keys, &read.errors).unwrap();
    assert!(matches!(
        serve(&source, span(4, 8)),
        Err(Error::NoPts { frame }) if frame == FrameIndex::new(6)
    ));
}

#[test]
fn a_timeline_whose_pts_fall_back_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let mut pts = all_pts(&read);
    pts.swap(3, 4);
    let timeline = forged(&read, pts);
    assert!(matches!(
        Source::new(&path, &timeline, &read.keys, &read.errors),
        Err(Error::UnorderedPts { frame }) if frame == FrameIndex::new(4)
    ));
}

#[test]
fn a_timeline_of_another_video_is_refused_at_the_first_unknown_pts() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let shifted = all_pts(&read)
        .into_iter()
        .map(|pts| pts.map(|pts| Pts::new(pts.get() + 1)))
        .collect();
    let timeline = forged(&read, shifted);
    let source = Source::new(&path, &timeline, &read.keys, &read.errors).unwrap();
    assert!(matches!(serve(&source, span(2, 3)), Err(Error::UnknownPts { .. })));
}

#[test]
fn a_frame_the_decoder_does_not_give_is_missing_rather_than_skipped() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let mut pts = all_pts(&read);
    let between = pts[9].unwrap().get() + 1;
    pts.insert(10, Some(Pts::new(between)));
    let timeline = forged(&read, pts);
    let source = Source::new(&path, &timeline, &read.keys, &read.errors).unwrap();
    assert!(matches!(
        serve(&source, span(8, 12)),
        Err(Error::Missing { frame }) if frame == FrameIndex::new(10)
    ));
}

#[test]
fn a_window_that_outlasts_the_file_is_missing_its_last_frames() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let mut pts = all_pts(&read);
    let after = pts.last().copied().flatten().unwrap().get() + 333;
    pts.push(Some(Pts::new(after)));
    let frames = pts.len();
    let timeline = forged(&read, pts);
    let source = Source::new(&path, &timeline, &read.keys, &read.errors).unwrap();
    assert!(matches!(
        serve(&source, span(frames - 3, frames - 1)),
        Err(Error::Missing { frame }) if frame == FrameIndex::new(frames - 1)
    ));
}

#[test]
fn a_visitor_error_stops_the_reading_and_comes_back_as_is() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    let mut visited = 0;
    let stopped = source.serve(span(2, 6), |_, _| {
        visited += 1;
        if visited == 2 { Err(Stop::Visitor) } else { Ok(()) }
    });
    assert!(matches!(stopped, Err(Stop::Visitor)));
    assert_eq!(visited, 2);
}

#[test]
fn an_unreadable_file_is_a_read_error_that_names_it() {
    let folder = tempfile::tempdir().unwrap();
    let (_, read) = losing_frames(folder.path());
    let missing = folder.path().join("missing.mp4");
    let source = Source::new(&missing, &read.timeline, &read.keys, &read.errors).unwrap();
    let error = source.serve(span(0, 1), |_, _| Ok::<(), Stop>(())).unwrap_err();
    let Stop::Source(error @ Error::Read { .. }) = error else {
        panic!("{error:?}");
    };
    assert!(error.to_string().contains("missing.mp4"));
}

#[test]
fn one_source_serves_windows_from_several_threads_at_once() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = losing_frames(folder.path());
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    let windows = [span(2, 9), span(14, 21), span(30, 37)];
    let served: Vec<Vec<u64>> = thread::scope(|scope| {
        let threads: Vec<_> = windows
            .iter()
            .map(|window| scope.spawn(|| serve(&source, *window).unwrap().1))
            .collect();
        threads
            .into_iter()
            .map(|thread| thread.join().unwrap().into_iter().map(|(_, pixels)| pixels).collect())
            .collect()
    });
    for (window, pixels) in windows.iter().zip(served) {
        assert_eq!(pixels, read.fingerprints[window.first().get()..=window.last().get()]);
    }
}

#[test]
fn a_reading_through_a_frame_in_error_takes_one_thread_and_a_clean_one_takes_them_all() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::binoculars(folder.path());
    let read = sequential::read(&path);
    let errors = [FrameIndex::new(20)];
    let source = Source::new(&path, &read.timeline, &read.keys, &errors).unwrap();
    let one = Threads::Exactly(std::num::NonZeroUsize::MIN);
    assert_eq!(source.threads(FrameIndex::new(16), FrameIndex::new(24)), one);
    assert_eq!(source.threads(FrameIndex::new(20), FrameIndex::new(20)), one);
    assert_eq!(
        source.threads(FrameIndex::new(21), FrameIndex::new(40)),
        Threads::Automatic
    );
    assert_eq!(
        source.threads(FrameIndex::new(0), FrameIndex::new(19)),
        Threads::Automatic
    );
}

fn pieces_of(errors: &[usize], pts: impl Fn(&Sequential) -> Vec<Option<Pts>>) -> (Vec<Span>, Source, Sequential) {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::binoculars(folder.path());
    let read = sequential::read(&path);
    let errors: Vec<FrameIndex> = errors.iter().copied().map(FrameIndex::new).collect();
    let source = Source::new(&path, &forged(&read, pts(&read)), &read.keys, &errors).unwrap();
    (source.pieces(), source, read)
}

#[test]
fn a_clean_video_is_read_as_a_single_piece() {
    let (pieces, _, _) = pieces_of(&[], all_pts);
    assert_eq!(pieces, [span(0, 47)]);
}

#[test]
fn a_frame_in_error_is_read_alone_from_the_key_frame_before_it_to_the_key_frame_after_it() {
    let (pieces, _, _) = pieces_of(&[19, 21], all_pts);
    assert_eq!(pieces, [span(0, 15), span(16, 23), span(24, 47)]);
}

#[test]
fn a_frame_without_pts_is_left_out_of_every_piece_and_of_the_stamps() {
    let without_30 = |read: &Sequential| {
        let mut pts = all_pts(read);
        pts[30] = None;
        pts
    };
    let (pieces, source, read) = pieces_of(&[], without_30);
    assert_eq!(pieces, [span(0, 29), span(31, 47)]);
    let stamps = source.stamps(&pieces);
    let expected: Vec<Pts> = (0..48)
        .filter(|frame| *frame != 30)
        .map(|frame| read.timeline.pts(FrameIndex::new(frame)).unwrap())
        .collect();
    assert_eq!(stamps, expected);
    assert_eq!(source.time_base(), read.timeline.time_base());
}
