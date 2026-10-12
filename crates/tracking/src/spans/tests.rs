use proptest::prelude::*;
use types::frame::FrameIndex;
use types::span::Span;

use super::coverage;
use crate::fake::span;

#[test]
fn a_span_covers_both_of_its_ends() {
    let covered = coverage(&[span(3, 5), span(9, 9)], 10);
    let frames: Vec<usize> = (0..10).filter(|&frame| covered[frame]).collect();
    assert_eq!(frames, [3, 4, 5, 9]);
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
