use proptest::prelude::*;
use types::detection::FrameDetections;
use types::frame::FrameIndex;
use types::span::Span;

use super::{coverage, without_spans};
use crate::fake::{crowd, span};

fn counts(detections: &FrameDetections) -> Vec<usize> {
    detections.iter().map(|(_, found)| found.len()).collect()
}

fn kept(counts: &[usize], spans: &[Span]) -> FrameDetections {
    without_spans(&crowd(counts), &coverage(spans, counts.len())).unwrap()
}

#[test]
fn ignored_frames_keep_their_place_with_no_detection() {
    assert_eq!(counts(&kept(&[1, 2, 3, 4, 5], &[span(1, 2)])), [1, 0, 0, 4, 5]);
}

#[test]
fn a_span_covers_both_of_its_ends() {
    let covered = coverage(&[span(3, 5), span(9, 9)], 10);
    let frames: Vec<usize> = (0..10).filter(|&frame| covered[frame]).collect();
    assert_eq!(frames, [3, 4, 5, 9]);
}

#[test]
fn many_short_spans_empty_exactly_their_frames() {
    let spans: Vec<Span> = (0..10_000).step_by(4).map(|first| span(first, first + 1)).collect();
    let left = kept(&vec![1; 10_000], &spans);
    let first_left: Vec<usize> = left
        .iter()
        .filter(|(_, found)| !found.is_empty())
        .map(|(frame, _)| frame.get())
        .take(4)
        .collect();
    assert_eq!(first_left, [2, 3, 6, 7]);
    assert_eq!(counts(&left).iter().filter(|&&count| count == 0).count(), 5_000);
}

#[test]
fn a_span_running_past_the_last_frame_stops_there() {
    assert_eq!(coverage(&[span(3, 50)], 5), [false, false, false, true, true]);
    assert_eq!(coverage(&[span(7, 9)], 5), [false; 5]);
    assert_eq!(coverage(&[span(3, 4)], 0), Vec::<bool>::new());
}

proptest! {
    #[test]
    fn a_frame_is_covered_exactly_when_some_span_contains_it(
        bounds in prop::collection::vec((0_usize..40, 0_usize..10), 0..6),
        frame_count in 0_usize..50,
    ) {
        let spans: Vec<Span> = bounds.iter().map(|&(first, length)| span(first, first + length)).collect();
        let covered = coverage(&spans, frame_count);
        prop_assert_eq!(covered.len(), frame_count);
        for (frame, &is_covered) in covered.iter().enumerate() {
            prop_assert_eq!(is_covered, spans.iter().any(|span| span.contains(FrameIndex::new(frame))));
        }
    }
}
