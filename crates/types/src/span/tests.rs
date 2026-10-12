use super::{Error, Span};
use crate::frame::FrameIndex;

fn span(first: usize, last: usize) -> Result<Span, Error> {
    Span::new(FrameIndex::new(first), FrameIndex::new(last))
}

#[test]
fn both_bounds_belong_to_the_span() {
    let damaged = span(1286, 1328).unwrap();
    assert!(damaged.contains(FrameIndex::new(1286)));
    assert!(damaged.contains(FrameIndex::new(1328)));
    assert!(!damaged.contains(FrameIndex::new(1285)));
    assert!(!damaged.contains(FrameIndex::new(1329)));
    assert_eq!(damaged.frame_count(), 43);
}

#[test]
fn a_single_frame_is_a_span_of_one() {
    let single = span(7, 7).unwrap();
    assert_eq!(
        (single.first(), single.last(), single.frame_count()),
        (FrameIndex::new(7), FrameIndex::new(7), 1)
    );
}

#[test]
fn a_span_ending_before_it_starts_is_refused() {
    let error = span(10, 9).unwrap_err();
    assert_eq!(
        error,
        Error::Reversed {
            first: FrameIndex::new(10),
            last: FrameIndex::new(9)
        }
    );
    assert_eq!(
        error.to_string(),
        "the span ends at frame 9 before it starts at frame 10"
    );
}

fn frames(span: Span) -> (usize, usize) {
    (span.first().get(), span.last().get())
}

#[test]
fn a_single_frame_span_needs_no_check() {
    assert_eq!(Span::single(FrameIndex::new(7)), span(7, 7).unwrap());
}

#[test]
fn the_hull_covers_both_spans_and_the_gap_between_them() {
    let early = span(3, 5).unwrap();
    let late = span(9, 12).unwrap();
    assert_eq!(frames(early.hull(late)), (3, 12));
    assert_eq!(frames(late.hull(early)), (3, 12));
    assert_eq!(frames(span(3, 20).unwrap().hull(late)), (3, 20));
}

#[test]
fn widening_stops_at_the_first_frame_and_at_the_last_index() {
    assert_eq!(frames(span(10, 12).unwrap().widened(4, 30)), (6, 42));
    assert_eq!(frames(span(2, 2).unwrap().widened(5, 0)), (0, 2));
    assert_eq!(frames(span(2, usize::MAX - 1).unwrap().widened(0, 5)), (2, usize::MAX));
}

#[test]
fn spans_sharing_frames_intersect_on_them() {
    let window = span(0, 99).unwrap();
    assert_eq!(span(95, 110).unwrap().intersection(window).map(frames), Some((95, 99)));
    assert_eq!(span(99, 99).unwrap().intersection(window).map(frames), Some((99, 99)));
}

#[test]
fn spans_without_a_shared_frame_have_no_intersection() {
    assert_eq!(span(100, 105).unwrap().intersection(span(0, 99).unwrap()), None);
}

#[test]
fn the_widest_span_counts_as_many_frames_as_a_usize_holds_rather_than_overflowing() {
    assert_eq!(span(0, usize::MAX).unwrap().frame_count(), usize::MAX);
}
