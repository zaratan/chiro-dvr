use render::source::{Error, Origin, Source};
use types::frame::FrameIndex;
use types::span::Span;

use crate::sequential::{self, Sequential};

#[cfg(test)]
pub(crate) fn assert_served_like_a_sequential_reading(source: &Source, read: &Sequential, window: Span) -> Origin {
    let mut served = Vec::new();
    let origin = source
        .serve(window, |frame, pixels| {
            served.push((frame, sequential::fingerprint(pixels)));
            Ok::<(), Error>(())
        })
        .unwrap();
    let expected: Vec<(FrameIndex, u64)> = (window.first().get()..=window.last().get())
        .map(|frame| (FrameIndex::new(frame), read.fingerprints[frame]))
        .collect();
    assert_eq!(served, expected, "{window:?}");
    origin
}
