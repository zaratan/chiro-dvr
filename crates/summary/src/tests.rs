use draw::canvas::{Canvas, PixelBox};
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
    let written = pages(&summary("v", &tracks, &timeline), gray(), &[], &DebugLabels)
        .unwrap()
        .pages;
    assert_eq!(written.len(), 1);
    assert_eq!(written[0].minutes, None);
    assert!(written[0].canvas.height() > 240);
}

#[test]
fn a_long_video_gives_one_page_per_ten_minutes() {
    let timeline = timeline(13_000);
    let written = pages(&summary("v", &[], &timeline), gray(), &[], &DebugLabels)
        .unwrap()
        .pages;
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
    let written = pages(&summary("v", &tracks, &timeline), gray(), &[], &DebugLabels)
        .unwrap()
        .pages;
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
    let written = pages(&summary("v", &tracks, &timeline), gray(), &[], &DebugLabels)
        .unwrap()
        .pages;
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

#[test]
fn the_backdrop_is_the_scene_of_the_complete_image_without_any_mark() {
    let timeline = timeline(40);
    let display = PixelBox {
        left: 0,
        top: 0,
        right: 99,
        bottom: 19,
    };
    let drawn = pages(&summary("v", &[], &timeline), gray(), &[display], &DebugLabels).unwrap();
    let (width, height) = (drawn.backdrop.width() as usize, drawn.backdrop.height() as usize);
    assert_eq!((width, height), (320, 240));
    let page = &drawn.pages[0].canvas;
    let page_row = page.width() as usize * 4;
    for row in 0..height {
        assert_eq!(
            &page.rgba()[row * page_row..row * page_row + width * 4],
            &drawn.backdrop.rgba()[row * width * 4..(row + 1) * width * 4],
            "{row}"
        );
    }
    assert_ne!(drawn.backdrop, gray());
}

#[test]
fn the_geometry_given_for_each_track_follows_its_points_and_puts_its_marker_where_it_is_drawn() {
    let points = [[40.0, 120.0], [120.0, 120.0], [200.0, 120.0]];
    let tracks = [track(3, 0, &points)];
    let timeline = timeline(40);
    let drawn = pages(&summary("v", &tracks, &timeline), gray(), &[], &DebugLabels).unwrap();
    let page = &drawn.pages[0];
    let painted = &page.painted[0];
    assert_eq!(painted.path, points);
    assert!(painted.head.is_some());
    let below_label = [painted.dot[0], painted.dot[1] + 0.75 * page.radius];
    let width = page.canvas.width() as usize;
    let at = (below_label[1] as usize * width + below_label[0] as usize) * 4;
    let color = painted.color;
    assert_eq!(
        &page.canvas.rgba()[at..at + 4],
        &[color.red, color.green, color.blue, 255]
    );
    assert_eq!(page.window, [0.0, 4.0]);
}
