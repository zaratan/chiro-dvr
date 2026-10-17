use super::render;
use crate::model::Layout;
use crate::testing::{Named, layout};

#[test]
fn a_window_without_any_lost_time_says_so_for_both_bands() {
    assert_eq!(
        render(&layout(), 14, &Named),
        "Scene(Window { start: 0.0, end: 300.07 }) · Tracks(14) · Scene(NothingNotAnalysed), Scene(NothingUnreadable)"
    );
}

#[test]
fn periods_are_rounded_out_to_whole_seconds_merged_and_folded_beyond_three() {
    let lost = Layout {
        ignored_s: vec![
            [1.2, 1.8],
            [1.9, 2.5],
            [10.0, 11.0],
            [20.0, 21.0],
            [30.0, 31.0],
            [40.0, 41.0],
        ],
        damaged_s: vec![[5.5, 6.25]],
        ..layout()
    };
    assert_eq!(
        render(&lost, 2, &Named),
        "Scene(Window { start: 0.0, end: 300.07 }) · Tracks(2) · Scene(NotAnalysed(&quot;Scene(Span { start: 1.0, end: 3.0 }), Scene(Span { start: 10.0, end: 11.0 }), Scene(Span { start: 20.0, end: 21.0 })&quot;)) Scene(More(2)), Scene(Unreadable(&quot;Scene(Span { start: 5.0, end: 7.0 })&quot;))"
    );
}
