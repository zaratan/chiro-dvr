use super::render;
use crate::testing::{Named, video};

#[test]
fn the_judging_area_waits_hidden_until_a_track_is_chosen() {
    let area = render(&video("v"), &Named);
    assert!(
        area.trim_start()
            .starts_with(r#"<div class="judge" hidden tabindex="-1">"#)
    );
}

#[test]
fn the_three_verdicts_are_toggles_with_their_key_shown_but_not_read_aloud() {
    let area = render(&video("v"), &Named);
    assert_eq!(area.matches(r#"aria-pressed="false" aria-keyshortcuts="#).count(), 3);
    assert!(area.contains(r#"data-verdict="yes" aria-pressed="false" aria-keyshortcuts="Script(KeyYes)">Script(Yes)<kbd aria-hidden="true">Script(KeyYes)</kbd>"#));
    assert!(area.contains(r#"data-verdict="doubt""#));
}

#[test]
fn the_clip_loops_and_waits_for_a_click_before_loading() {
    let area = render(&video("v"), &Named);
    assert!(area.contains(r#"<video loop playsinline preload="none""#));
}

#[test]
fn the_steps_between_tracks_are_named_for_screen_readers() {
    let area = render(&video("v"), &Named);
    assert!(area.contains(r#"aria-label="Script(PreviousTrack)" aria-keyshortcuts="ArrowLeft">‹"#));
    assert!(area.contains(r#"aria-label="Script(NextTrack)" aria-keyshortcuts="ArrowRight">›"#));
}
