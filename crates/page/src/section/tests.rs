use super::{render, render_previous};
use crate::model::TooMany;
use crate::testing::{Named, drawn_video, summary, track, video};

#[test]
fn a_video_with_one_summary_image_has_no_tabs() {
    let section = render(&video("v"), "v1", &Named);
    assert!(!section.contains("role=\"tab\""));
    assert_eq!(section.matches(r#"<div class="panel">"#).count(), 1);
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
fn a_folded_video_with_too_many_tracks_still_says_so_in_its_heading() {
    let mut night = video("v");
    assert!(!render(&night, "v1", &Named).contains("TooManyBadge"));
    night.too_many = Some(TooMany { count: 301, limit: 300 });
    night.open = false;
    let section = render(&night, "v1", &Named);
    let (heading, _) = section.split_once("</summary>").unwrap();
    assert!(heading.contains(r#"<span class="badge">TooManyBadge</span>"#));
}

#[test]
fn a_video_of_the_last_run_opens_and_one_of_an_earlier_evening_stays_folded() {
    let mut night = video("v");
    assert!(render(&night, "v1", &Named).starts_with(r#"<details class="video" id="v1" data-video="v" open>"#));
    night.open = false;
    assert!(render(&night, "v1", &Named).starts_with(r#"<details class="video" id="v1" data-video="v">"#));
}

#[test]
fn the_results_kept_from_before_a_failure_stay_folded_and_say_where_they_come_from() {
    let section = render_previous(&video("v"), "v3-previous", &Named);
    assert!(section.starts_with(r#"<details class="video" id="v3-previous" data-video="v">"#));
    assert!(section.contains("PreviousResults"));
}

#[test]
fn a_section_names_its_folder_so_the_verdicts_find_their_video() {
    let section = render(&video("nuit <12>"), "v1", &Named);
    assert!(section.contains(r#"data-video="nuit &lt;12&gt;""#));
}

#[test]
fn the_judging_area_replaces_the_picture_in_the_same_column() {
    let section = render(&video("v"), "v1", &Named);
    let stage = section.split(r#"<div class="review">"#).next().unwrap();
    assert!(stage.contains(r#"<div class="judge" hidden"#));
    assert!(section.contains(r#"<a class="thumb" hidden target="_blank" aria-label="Scene(CompleteNewTab)">"#));
    assert!(section.contains(r#"<div class="orphans" hidden></div>"#));
}

#[test]
fn a_video_written_with_its_background_and_layout_is_drawn_again_with_its_line_its_link_and_its_strip() {
    let section = render(&drawn_video("v"), "v1", &Named);
    let line = section
        .find(r#"<p class="scene-line"><span><span aria-hidden="true">Scene(Window"#)
        .unwrap();
    let scene = section.find(r#"<svg class="scene""#).unwrap();
    let strip = section.find(r#"<div class="strip" aria-hidden="true">"#).unwrap();
    assert!(line < scene && scene < strip, "{section}");
    assert!(section.contains(r#"<a class="complete" href="v/v_resume.png" target="_blank">Scene(Complete)<span class="sr"> Scene(NewTab)</span></a>"#));
    assert!(section.contains(r#"aria-label="Scene(Alt { video: &quot;v&quot;, tracks: 4 })""#));
    assert_eq!(section.matches(r#"<g data-id=""#).count(), 4);
    assert!(!section.contains("<img src="));
}

#[test]
fn a_folder_written_before_the_background_existed_keeps_its_complete_image_without_strip() {
    let mut older = drawn_video("v");
    older.background = None;
    let section = render(&older, "v1", &Named);
    assert!(section.contains(r#"<img src="v/v_resume.png""#), "{section}");
    assert!(!section.contains("<svg") && !section.contains(r#"class="strip""#));
    let mut unlaid = drawn_video("v");
    unlaid.summaries[0].layout = None;
    assert!(render(&unlaid, "v1", &Named).contains(r#"<img src="v/v_resume.png""#));
}

#[test]
fn each_part_draws_only_its_own_tracks() {
    let mut night = drawn_video("v");
    night.summaries = vec![summary("v_1.png"), summary("v_2.png")];
    for summary in &mut night.summaries {
        summary.layout = drawn_video("v").summaries[0].layout.clone();
    }
    for track in &mut night.tracks {
        track.summary = Some(if track.id <= 3 { "v_1.png" } else { "v_2.png" }.to_owned());
    }
    let section = render(&night, "v1", &Named);
    let (first, second) = section.split_once(r#"id="v1-p2""#).unwrap();
    assert_eq!(first.matches(r#"<g data-id=""#).count(), 3);
    assert_eq!(second.matches(r#"<g data-id=""#).count(), 1);
    assert!(second.contains("Scene(PartAlt"));
}

#[test]
fn a_video_without_tracks_still_draws_its_background_its_line_and_an_empty_strip() {
    let mut quiet = drawn_video("v");
    quiet.tracks.clear();
    let section = render(&quiet, "v1", &Named);
    assert!(
        section.contains(r#"<span aria-hidden="true">Scene(Window { start: &quot;Scene(Clock(0.0))&quot;, end: &quot;Scene(Clock(300.07))&quot; }) · Tracks(0) ·"#),
        "{section}"
    );
    assert!(section.contains(r#"<image href="v/v_fond.png""#));
    assert!(section.contains(r#"<div class="lane starts"></div>"#));
    assert!(!section.contains("data-id="));
}

fn crowded_without_clips_or_picture() -> crate::model::Video {
    let mut crowded = video("v");
    crowded.too_many = Some(TooMany { count: 301, limit: 300 });
    crowded.summaries.clear();
    for track in &mut crowded.tracks {
        track.summary = None;
        track.clip = None;
        track.zoom = None;
    }
    crowded
}

#[test]
fn a_video_with_neither_clip_nor_picture_has_no_viewer_and_its_track_numbers_are_not_buttons() {
    let section = render(&crowded_without_clips_or_picture(), "v1", &Named);
    assert!(section.starts_with(r#"<details class="video no-viewer""#));
    assert!(!section.contains(r#"class="judge""#));
    assert!(!section.contains(r#"<button type="button" class="dot""#));
    assert_eq!(section.matches(r#"<span class="dot""#).count(), 4);
    assert!(!section.contains(r#"class="verdict""#));
}

#[test]
fn a_single_clip_is_enough_for_the_viewer_of_a_video_without_picture() {
    let mut crowded = crowded_without_clips_or_picture();
    crowded.tracks[2].clip = Some("extraits/piste_03_0m03s.mp4".to_owned());
    let section = render(&crowded, "v1", &Named);
    assert!(section.starts_with(r#"<details class="video" "#));
    assert!(section.contains(r#"<div class="judge" hidden"#));
    assert_eq!(section.matches(r#"<button type="button" class="dot""#).count(), 4);
}
