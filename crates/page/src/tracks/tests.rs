use super::render;
use crate::testing::{Named, summary, track, video};

#[test]
fn each_row_carries_the_key_that_ties_a_verdict_to_its_track() {
    let mut night = video("v");
    night.tracks[0].start_s = 72.394;
    night.tracks[0].start_xy = Some([959, 644]);
    let table = render(&night, true, &Named);
    assert!(table.contains(r#"<tr data-id="1" data-start="72.39" data-x="959" data-y="644""#));
}

#[test]
fn a_track_written_before_the_starting_points_has_no_position() {
    let table = render(&video("v"), true, &Named);
    assert!(table.contains(r#"<tr data-id="2" data-start="6.80" data-clip="#));
    assert!(!table.contains("data-x"));
}

#[test]
fn a_row_names_its_clip_and_its_zoom_only_when_they_were_written() {
    let mut night = video("v");
    night.tracks[1].zoom = Some("extraits/piste_02_0m05s_zoom.mp4".to_owned());
    let table = render(&night, true, &Named);
    assert_eq!(table.matches(" data-clip=").count(), 4);
    assert_eq!(
        table
            .matches(r#" data-zoom="v/extraits/piste_02_0m05s_zoom.mp4""#)
            .count(),
        1
    );
    assert_eq!(table.matches(r#"<span class="zoom-tag">"#).count(), 1);
}

#[test]
fn every_track_starts_to_be_seen_until_the_script_reads_the_verdicts() {
    let table = render(&video("v"), true, &Named);
    assert_eq!(
        table.matches(r#"<span class="chip todo">Script(ToSee)</span>"#).count(),
        4
    );
}

#[test]
fn the_number_of_a_track_is_the_button_that_opens_it_for_judging() {
    let table = render(&video("v"), true, &Named);
    assert!(table.contains(r#"aria-label="Script(JudgeTrack)">1</button>"#));
}

#[test]
fn each_row_names_the_part_of_the_summary_image_that_draws_its_track() {
    let mut night = video("v");
    night.summaries = vec![summary("v_1.png"), summary("v_2.png")];
    night.tracks = vec![track(1, "v_1.png"), track(2, "v_2.png")];
    let table = render(&night, true, &Named);
    assert!(table.contains(r#" data-part="1">"#));
    assert!(table.contains(r#" data-part="2">"#));
}

#[test]
fn the_dot_takes_the_colour_of_the_path_on_the_image() {
    let table = render(&video("v"), true, &Named);
    assert!(table.contains(r#"style="--dot: rgb(255 214 10)""#));
}

#[test]
fn the_links_to_the_table_and_the_clips_are_relative_and_encoded() {
    let night = video("nuit 12 #1");
    let table = render(&night, true, &Named);
    assert!(table.contains(r#"href="nuit%2012%20%231/v_pistes.csv""#));
    assert!(table.contains(r#"data-clip="nuit%2012%20%231/extraits/piste_01_0m04s.mp4""#));
}

#[test]
fn a_table_without_viewer_has_only_the_number_start_and_length_of_each_track() {
    let night = video("v");
    let table = render(&night, false, &Named);
    assert!(!table.contains("<button"));
    assert!(!table.contains("ColumnVerdict") && !table.contains("c-zoom"));
    assert!(!table.contains(r#"class="verdict""#) && !table.contains(r#"class="zoom""#));
    assert_eq!(table.matches("<th scope=\"col\"").count(), 3);
    assert_eq!(table.matches("<td ").count(), 2 * night.tracks.len());
}

#[test]
fn a_table_with_viewer_has_a_verdict_and_a_zoom_column_for_each_track() {
    let night = video("v");
    let table = render(&night, true, &Named);
    assert_eq!(table.matches("<th scope=\"col\"").count(), 5);
    assert_eq!(table.matches(r#"<td class="verdict">"#).count(), night.tracks.len());
    assert_eq!(table.matches(r#"<td class="zoom">"#).count(), night.tracks.len());
}
