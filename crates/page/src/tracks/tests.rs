use super::render;
use crate::testing::{Named, track, video};

#[test]
fn a_zoom_button_appears_only_for_a_track_whose_zoom_was_written() {
    let mut night = video("v");
    night.tracks[1].zoom = Some("extraits/piste_02_0m05s_zoom.mp4".to_owned());
    let table = render(&night, &Named);
    assert_eq!(table.matches("PlayClip(").count(), 4);
    assert_eq!(table.matches(r#"aria-label="PlayZoom(2)""#).count(), 1);
    assert!(table.contains(r#"data-src="v/extraits/piste_02_0m05s_zoom.mp4""#));
}

#[test]
fn a_track_without_clips_has_no_button_to_play_but_can_still_be_shown_on_the_image() {
    let mut night = video("v");
    for one in &mut night.tracks {
        one.clip = None;
    }
    let table = render(&night, &Named);
    assert!(!table.contains("class=\"play\""));
    assert_eq!(table.matches("ShowTrack(").count(), 4);
}

#[test]
fn a_verdict_read_in_the_table_is_shown_and_an_empty_one_is_announced_as_to_note() {
    let mut night = video("v");
    night.tracks[0].verdict = Some("chauve-souris <oreillard>".to_owned());
    night.tracks[1].verdict = Some("  ".to_owned());
    let table = render(&night, &Named);
    assert!(table.contains(r#"<td class="verdict">chauve-souris &lt;oreillard&gt;</td>"#));
    assert_eq!(table.matches("NoVerdict").count(), 3);
}

#[test]
fn each_row_names_the_part_of_the_summary_image_that_draws_its_track() {
    let mut night = video("v");
    night.summaries = vec![crate::testing::summary("v_1.png"), crate::testing::summary("v_2.png")];
    night.tracks = vec![track(1, "v_1.png"), track(2, "v_2.png")];
    let table = render(&night, &Named);
    assert!(table.contains(r#"<tr data-part="1">"#));
    assert!(table.contains(r#"<tr data-part="2">"#));
}

#[test]
fn the_dot_takes_the_colour_of_the_path_on_the_image() {
    let table = render(&video("v"), &Named);
    assert!(table.contains(r#"style="--dot: rgb(255 214 10)""#));
}

#[test]
fn the_links_to_the_table_and_the_clips_are_relative_and_encoded() {
    let night = video("nuit 12 #1");
    let table = render(&night, &Named);
    assert!(table.contains(r#"href="nuit%2012%20%231/v_pistes.csv""#));
    assert!(table.contains(r#"data-src="nuit%2012%20%231/extraits/piste_01_0m04s.mp4""#));
}
