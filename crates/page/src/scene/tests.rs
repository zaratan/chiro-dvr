use super::{Scene, render};
use crate::model::{Background, Track};
use crate::testing::{trace, track};

const LAYERS: [&str; 5] = ["outlines", "lines", "heads", "leaders", "dots"];

fn background() -> Background {
    Background {
        file: "v fond.png".to_owned(),
        width: 1440,
        height: 1080,
    }
}

fn drawn(id: usize) -> Track {
    Track {
        trace: Some(trace(id)),
        ..track(id, "v_resume.png")
    }
}

fn scene(tracks: &[&Track]) -> String {
    render(Scene {
        folder: "nuit 1",
        background: &background(),
        radius: 23.0,
        tracks,
        alt: "Image \"résumé\"",
    })
}

fn layer<'a>(svg: &'a str, class: &str) -> &'a str {
    svg.split(r#"<g class=""#)
        .find(|chunk| chunk.starts_with(&format!(r#"{class}""#)))
        .unwrap()
}

#[test]
fn the_drawing_is_in_pixels_of_the_video_on_its_background_linked_from_the_folder() {
    let svg = scene(&[]);
    assert!(svg.starts_with(r#"<svg class="scene" viewBox="0 0 1440 1080" width="1440" height="1080" role="img" aria-label="Image &quot;résumé&quot;">"#), "{svg}");
    assert!(svg.contains(r#"<image href="nuit%201/v%20fond.png" width="1440" height="1080"/>"#));
}

#[test]
fn each_track_is_drawn_once_in_each_layer_and_the_layers_follow_the_complete_image() {
    let (three, seven) = (drawn(3), drawn(7));
    let svg = scene(&[&three, &seven]);
    let starts: Vec<usize> = LAYERS
        .iter()
        .map(|class| svg.find(&format!(r#"<g class="{class}""#)).unwrap())
        .collect();
    assert!(starts.is_sorted(), "{svg}");
    for class in ["outlines", "lines", "heads", "dots"] {
        let shapes = layer(&svg, class);
        assert_eq!(shapes.matches(r#"data-id="3""#).count(), 1, "{class}");
        assert_eq!(shapes.matches(r#"data-id="7""#).count(), 1, "{class}");
        assert!(shapes.find(r#"data-id="3""#) < shapes.find(r#"data-id="7""#), "{class}");
    }
    assert!(
        layer(&svg, "lines").contains(r#"<polyline data-id="3" points="10,300 200,300" stroke="rgb(255 214 10)"/>"#)
    );
    assert!(
        layer(&svg, "dots")
            .contains(r#"<circle cx="250" cy="300" r="23.0" fill="rgb(255 214 10)"/><text x="250" y="300">3</text>"#)
    );
    assert!(svg.contains(r#"<g class="dots" font-size="25.3">"#));
}

#[test]
fn a_leader_joins_only_a_dot_moved_away_from_its_trace() {
    let mut moved = drawn(3);
    if let Some(trace) = moved.trace.as_mut() {
        trace.anchor = Some([200, 300]);
    }
    let kept = drawn(7);
    let svg = scene(&[&moved, &kept]);
    let leaders = layer(&svg, "leaders");
    assert!(
        leaders.contains(r#"<g data-id="3"><line class="halo" x1="200" y1="300" x2="250" y2="300"/>"#),
        "{leaders}"
    );
    assert!(!leaders.contains(r#"data-id="7""#));
}

#[test]
fn a_trace_edited_by_hand_cannot_close_its_attribute() {
    let mut forged = drawn(1);
    if let Some(trace) = forged.trace.as_mut() {
        trace.points = r#"1,1"/><script>alert(1)</script>"#.to_owned();
        trace.head = Some(r#"2,2"><b>"#.to_owned());
    }
    let svg = scene(&[&forged]);
    assert!(!svg.contains("<script") && !svg.contains("<b>"), "{svg}");
    assert!(svg.contains(r#"points="1,1&quot;/&gt;&lt;script&gt;alert(1)&lt;/script&gt;""#));
}

#[test]
fn a_track_without_trace_or_colour_is_left_out_of_the_drawing() {
    let bare = track(1, "v_resume.png");
    let grey = Track {
        color: None,
        ..drawn(2)
    };
    let svg = scene(&[&bare, &grey]);
    assert!(!svg.contains("data-id"), "{svg}");
}
