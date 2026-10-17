use super::{Strip, render};
use crate::model::{Layout, Track};
use crate::testing::{Named, layout, track};

fn strip(layout: &Layout, tracks: &[&Track]) -> String {
    render(Strip { layout, tracks }, &Named)
}

fn ticks(html: &str) -> Vec<String> {
    html.split(r#"<span class="tick"#)
        .skip(1)
        .map(|tick| tick.split('"').next().unwrap_or_default().to_owned())
        .collect()
}

#[test]
fn a_band_shows_only_when_it_holds_a_period() {
    let html = strip(&layout(), &[]);
    assert!(!html.contains("lane-name\">Scene(Band"), "{html}");
    let damaged = Layout {
        damaged_s: vec![[30.007, 60.014]],
        ..layout()
    };
    let html = strip(&damaged, &[]);
    assert!(html.contains(r#"<span class="lane-name">Scene(BandUnreadable)</span><div class="lane lost damaged"><span style="left:10.000%;width:10.000%"></span></div>"#), "{html}");
    assert!(!html.contains("Scene(BandNotAnalysed)"));
}

#[test]
fn the_starts_of_the_tracks_take_their_colour_and_those_of_the_other_parts_come_first_in_grey() {
    let one = track(1, "v_resume.png");
    let other = Layout {
        other_starts_s: vec![150.035],
        ..layout()
    };
    let html = strip(&other, &[&one]);
    let grey = html
        .find(r#"<span class="start other" style="left:50.000%"></span>"#)
        .unwrap();
    let mine = html.find(r#"<span class="start" style="left:1.933%;--dot:rgb(255 214 10)" title="Scene(StartOf { track: 1, start: 5.8 })"></span>"#).unwrap();
    assert!(grey < mine, "{html}");
}

#[test]
fn each_minute_is_marked_with_every_step_it_belongs_to_and_the_ends_keep_their_label_inside() {
    let ten_minutes = Layout {
        window_s: [0.0, 600.0],
        ..layout()
    };
    let html = strip(&ten_minutes, &[]);
    let found = ticks(&html);
    assert_eq!(found.len(), 11);
    assert_eq!(found[0], " m2 m5 m10 first");
    assert_eq!(found[1], "");
    assert_eq!(found[4], " m2");
    assert_eq!(found[5], " m5");
    assert_eq!(found[10], " m2 m5 m10 last");
    assert!(html.contains(r#"<div class="unit" style="width:10.000%"><span class="tick m2 m5 m10 first" style="left:0.000%">Scene(Clock(0.0))</span>"#), "{html}");
    assert!(html.contains(r#"style="left:1000.000%">Scene(Clock(600.0))</span>"#));
}

#[test]
fn the_window_of_a_later_period_starts_its_minutes_where_it_starts() {
    let later = Layout {
        window_s: [600.0, 900.07],
        ..layout()
    };
    let html = strip(&later, &[]);
    assert_eq!(ticks(&html).len(), 6);
    assert!(
        html.contains(r#"<span class="tick m2 m5 m10 first" style="left:0.000%">Scene(Clock(600.0))</span>"#),
        "{html}"
    );
    assert!(html.contains(r#"<div class="unit" style="width:19.995%">"#));
}

#[test]
fn a_window_without_length_draws_no_strip() {
    let empty = Layout {
        window_s: [0.0, 0.0],
        ..layout()
    };
    assert_eq!(strip(&empty, &[]), "");
}
