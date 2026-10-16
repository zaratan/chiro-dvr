use draw::canvas::Canvas;
use draw::color::{PALETTE, Rgb};

use super::{Painted, pages};
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
    assert_eq!(written[0].minutes, None);
    assert!(written[0].canvas.height() > 240);
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

#[test]
fn each_track_is_painted_once_under_its_number_in_a_palette_colour_found_on_the_image() {
    let tracks = [
        track(3, 0, &[[40.0, 120.0], [120.0, 120.0], [200.0, 120.0]]),
        track(7, 5, &[[40.0, 130.0], [120.0, 130.0], [200.0, 130.0]]),
    ];
    let timeline = timeline(40);
    let written = pages(&summary("v", &tracks, &timeline), gray(), &[], &DebugLabels).unwrap();
    let painted: Vec<Painted> = written.iter().flat_map(|page| page.painted.clone()).collect();
    assert_eq!(painted.iter().map(|one| one.track).collect::<Vec<_>>(), [3, 7]);
    assert_ne!(painted[0].color, painted[1].color);
    for Painted { color, .. } in painted {
        assert!(PALETTE.contains(&color), "{color:?}");
        let pixel = [color.red, color.green, color.blue, 255];
        assert!(
            written[0].canvas.rgba().as_chunks::<4>().0.contains(&pixel),
            "{color:?}"
        );
    }
}

#[test]
fn a_crowded_video_paints_each_track_on_exactly_one_of_its_parts() {
    let tracks: Vec<types::track::Track> = (1..=45)
        .map(|id| {
            let y = 10.0 + 4.5 * id as f64;
            track(id, id * 5, &[[40.0, y], [120.0, y], [200.0, y]])
        })
        .collect();
    let timeline = timeline(400);
    let written = pages(&summary("v", &tracks, &timeline), gray(), &[], &DebugLabels).unwrap();
    assert_eq!(written.len(), 3);
    let mut painted: Vec<usize> = written
        .iter()
        .flat_map(|page| page.painted.iter().map(|one| one.track))
        .collect();
    assert!(
        written
            .iter()
            .all(|page| page.part.is_some() && !page.painted.is_empty())
    );
    painted.sort_unstable();
    assert_eq!(painted, (1..=45).collect::<Vec<_>>());
}
