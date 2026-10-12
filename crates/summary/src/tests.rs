use draw::canvas::Canvas;
use draw::color::Rgb;

use super::pages;
use crate::layout::Summary;
use crate::testing::{DebugLabels, timeline, track};

const GRAY: Rgb = Rgb::new(128, 128, 128);

fn gray() -> Canvas {
    Canvas::new(320, 240, GRAY).unwrap()
}

fn summary<'a>(
    name: &'a str,
    tracks: &'a [types::track::Track],
    timeline: &'a types::timeline::Timeline,
) -> Summary<'a> {
    Summary {
        name,
        tracks,
        timeline,
        ignored: &[],
        damaged: &[],
    }
}

#[test]
fn a_short_video_gives_a_single_page_without_minutes() {
    let tracks = [track(1, 0, &[[40.0, 200.0], [50.0, 195.0], [60.0, 190.0]])];
    let timeline = timeline(40);
    let written = pages(&summary("v", &tracks, &timeline), gray(), &[], &DebugLabels).unwrap();
    assert_eq!(written.len(), 1);
    assert_eq!((written[0].minutes.clone(), written[0].canvas.height()), (None, 240));
}

#[test]
fn a_long_video_gives_one_page_per_ten_minutes() {
    let timeline = timeline(13_000);
    let written = pages(&summary("v", &[], &timeline), gray(), &[], &DebugLabels).unwrap();
    let minutes: Vec<_> = written.into_iter().map(|page| page.minutes).collect();
    assert_eq!(minutes, [Some(0..10), Some(10..20), Some(20..22)]);
}

#[test]
fn a_decomposed_accented_video_name_is_written_with_the_same_pixels_as_the_composed_one() {
    let timeline = timeline(40);
    let composed = pages(&summary("vidéo_été", &[], &timeline), gray(), &[], &DebugLabels).unwrap();
    let decomposed = pages(
        &summary("vide\u{301}o_e\u{301}te\u{301}", &[], &timeline),
        gray(),
        &[],
        &DebugLabels,
    )
    .unwrap();
    assert_eq!(composed, decomposed);
    let plain = pages(&summary("video_ete", &[], &timeline), gray(), &[], &DebugLabels).unwrap();
    assert_ne!(composed, plain);
}
