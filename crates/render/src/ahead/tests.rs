use std::path::Path;

use types::frame::FrameIndex;
use types::span::Span;

use super::serve_ahead;
use crate::clips::Error;
use crate::sequential::{self, Sequential};
use crate::source::{self, Origin, Source};
use crate::testing;

fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

fn video(folder: &Path) -> (Sequential, Source) {
    let path = testing::binoculars_of(folder, 64, 48);
    let read = sequential::read(&path);
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    (read, source)
}

#[test]
fn frames_read_ahead_arrive_once_in_order_with_the_pixels_of_a_sequential_reading() {
    let folder = tempfile::tempdir().unwrap();
    let (read, source) = video(folder.path());
    let mut served = Vec::new();
    let origin = serve_ahead(&source, span(10, 40), |frame, pixels| {
        served.push((frame.get(), sequential::fingerprint(pixels)));
        Ok(())
    })
    .unwrap();
    assert!(matches!(origin, Origin::Key(_)));
    let expected: Vec<(usize, u64)> = (10..=40).map(|frame| (frame, read.fingerprints[frame])).collect();
    assert_eq!(served, expected);
}

#[test]
fn a_consumer_error_stops_the_reading_and_comes_back_as_is() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = video(folder.path());
    let mut served = 0;
    let error = serve_ahead(&source, span(0, 40), |frame, _| {
        served += 1;
        if frame.get() == 5 { Err(Error::Stopped) } else { Ok(()) }
    })
    .unwrap_err();
    assert!(matches!(error, Error::Stopped));
    assert_eq!(served, 6);
}

#[test]
fn a_source_error_comes_back_as_a_source_error_before_any_frame() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = video(folder.path());
    let mut served = 0;
    let error = serve_ahead(&source, span(40, 60), |_, _| {
        served += 1;
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(error, Error::Source(source::Error::OutsideVideo { .. })));
    assert_eq!(served, 0);
}

#[test]
#[should_panic(expected = "visitor panic")]
fn a_panic_of_the_visitor_reaches_the_caller_without_leaving_the_reader_blocked() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = video(folder.path());
    let served = serve_ahead(&source, span(0, 40), |frame, _| {
        assert_ne!(frame.get(), 5, "visitor panic");
        Ok(())
    });
    drop(served);
}
