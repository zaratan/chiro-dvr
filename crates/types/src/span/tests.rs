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
