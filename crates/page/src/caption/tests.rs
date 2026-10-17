use super::render;
use crate::model::Layout;
use crate::testing::{Named, layout};

const SHOWN: &str = r#"<span aria-hidden="true">"#;
const SAID: &str = r#"</span><span class="sr">"#;

fn shown(caption: &str) -> &str {
    caption
        .strip_prefix(SHOWN)
        .and_then(|rest| rest.split_once(SAID))
        .map_or("", |(shown, _)| shown)
}

fn said(caption: &str) -> &str {
    caption
        .split_once(SAID)
        .and_then(|(_, rest)| rest.strip_suffix("</span>"))
        .unwrap_or("")
}

#[test]
fn a_window_without_any_lost_time_says_so_for_both_bands() {
    assert_eq!(
        shown(&render(&layout(), 14, &Named)),
        "Scene(Window { start: &quot;Scene(Clock(0.0))&quot;, end: &quot;Scene(Clock(300.07))&quot; }) · Tracks(14) · Scene(NothingNotAnalysed), Scene(NothingUnreadable)"
    );
}

#[test]
fn the_line_is_read_aloud_with_its_times_in_words_while_it_still_shows_the_clock() {
    assert_eq!(
        said(&render(&layout(), 14, &Named)),
        "Scene(Window { start: &quot;Aloud(0.0)&quot;, end: &quot;Aloud(300.07)&quot; }) · Tracks(14) · Scene(NothingNotAnalysed), Scene(NothingUnreadable)"
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
    let caption = render(&lost, 2, &Named);
    assert_eq!(
        shown(&caption),
        r"Scene(Window { start: &quot;Scene(Clock(0.0))&quot;, end: &quot;Scene(Clock(300.07))&quot; }) · Tracks(2) · Scene(NotAnalysed(&quot;Scene(Span { start: \&quot;Scene(Clock(1.0))\&quot;, end: \&quot;Scene(Clock(3.0))\&quot; }), Scene(Span { start: \&quot;Scene(Clock(10.0))\&quot;, end: \&quot;Scene(Clock(11.0))\&quot; }), Scene(Span { start: \&quot;Scene(Clock(20.0))\&quot;, end: \&quot;Scene(Clock(21.0))\&quot; })&quot;)) Scene(More(2)), Scene(Unreadable(&quot;Scene(Span { start: \&quot;Scene(Clock(5.0))\&quot;, end: \&quot;Scene(Clock(7.0))\&quot; })&quot;))"
    );
    assert_eq!(
        said(&caption),
        r"Scene(Window { start: &quot;Aloud(0.0)&quot;, end: &quot;Aloud(300.07)&quot; }) · Tracks(2) · Scene(NotAnalysed(&quot;Scene(SpanAloud { start: \&quot;Aloud(1.0)\&quot;, end: \&quot;Aloud(3.0)\&quot; }), Scene(SpanAloud { start: \&quot;Aloud(10.0)\&quot;, end: \&quot;Aloud(11.0)\&quot; }), Scene(SpanAloud { start: \&quot;Aloud(20.0)\&quot;, end: \&quot;Aloud(21.0)\&quot; })&quot;)) Scene(More(2)), Scene(Unreadable(&quot;Scene(SpanAloud { start: \&quot;Aloud(5.0)\&quot;, end: \&quot;Aloud(7.0)\&quot; })&quot;))"
    );
}
