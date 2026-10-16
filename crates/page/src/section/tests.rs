use super::{render, render_previous};
use crate::model::TooMany;
use crate::testing::{Named, summary, track, video};

#[test]
fn a_video_with_one_summary_image_has_no_tabs() {
    let section = render(&video("v"), "v1", &Named);
    assert!(!section.contains("role=\"tab\""));
    assert_eq!(section.matches("<img ").count(), 1);
}

#[test]
fn several_summary_images_get_one_tab_each_naming_its_tracks_and_only_the_first_shows() {
    let mut night = video("v");
    night.summaries = vec![summary("v_1.png"), summary("v_2.png"), summary("v_3.png")];
    night.tracks = vec![
        track(1, "v_1.png"),
        track(2, "v_1.png"),
        track(3, "v_2.png"),
        track(4, "v_3.png"),
    ];
    let section = render(&night, "v4", &Named);
    assert_eq!(section.matches("role=\"tab\"").count(), 3);
    assert!(section.contains(r#"<span class="part-range">1–2</span>"#));
    assert!(section.contains(r#"<span class="part-range">4–4</span>"#));
    assert_eq!(section.matches(r#"<div class="panel" role="tabpanel""#).count(), 3);
    assert_eq!(section.matches(" hidden><a").count(), 2);
}

#[test]
fn the_image_declares_its_size_so_a_tab_does_not_make_the_page_jump() {
    let section = render(&video("v"), "v1", &Named);
    assert!(section.contains(r#"width="1879" height="1215""#));
}

#[test]
fn the_time_not_analysed_shows_only_when_there_is_some() {
    let mut night = video("v");
    assert!(!render(&night, "v1", &Named).contains("NotAnalysed"));
    night.not_analysed_s = 32.2;
    assert!(render(&night, "v1", &Named).contains("NotAnalysed(33)"));
}

#[test]
fn a_video_with_too_many_tracks_says_so_above_its_image() {
    let mut night = video("v");
    night.too_many = Some(TooMany { count: 301, limit: 300 });
    let section = render(&night, "v1", &Named);
    assert!(section.contains("TooMany { video: &quot;v&quot;, count: 301, limit: 300 }"));
    assert!(section.contains("TooManyTodo"));
}

#[test]
fn a_video_of_the_last_run_opens_and_one_of_an_earlier_evening_stays_folded() {
    let mut night = video("v");
    assert!(render(&night, "v1", &Named).starts_with(r#"<details class="video" id="v1" open>"#));
    night.open = false;
    assert!(render(&night, "v1", &Named).starts_with(r#"<details class="video" id="v1">"#));
}

#[test]
fn the_results_kept_from_before_a_failure_stay_folded_and_say_where_they_come_from() {
    let section = render_previous(&video("v"), "v3-previous", &Named);
    assert!(section.starts_with(r#"<details class="video" id="v3-previous">"#));
    assert!(section.contains("PreviousOf(&quot;v&quot;)"));
}
