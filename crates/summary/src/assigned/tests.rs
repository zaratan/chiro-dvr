use draw::canvas::Canvas;
use draw::color::Rgb;
use types::track::Track;

use super::track_colors;
use crate::layout::Summary;
use crate::testing::{DebugLabels, timeline, track};
use crate::{Painted, pages};

#[test]
fn the_track_colours_are_the_ones_painted_on_the_summary_in_the_order_of_the_tracks() {
    let tracks: Vec<Track> = (1..=45)
        .rev()
        .map(|id| {
            let y = 10.0 + 4.5 * id as f64;
            track(id, id * 5, &[[40.0, y], [120.0, y], [200.0, y]])
        })
        .collect();
    let timeline = timeline(400);
    let summary = Summary {
        name: "v",
        tracks: &tracks,
        timeline: &timeline,
        ignored: &[],
        damaged: &[],
    };
    let written = pages(
        &summary,
        Canvas::new(320, 240, Rgb::new(128, 128, 128)).unwrap(),
        &[],
        &DebugLabels,
    )
    .unwrap();
    let painted: Vec<Painted> = written.pages.iter().flat_map(|page| page.painted.clone()).collect();
    let expected: Vec<Option<Rgb>> = tracks
        .iter()
        .map(|track| painted.iter().find(|one| one.track == track.id()).map(|one| one.color))
        .collect();
    assert_eq!(track_colors(&summary, 320, 240).unwrap(), expected);
    assert!(expected.iter().all(Option::is_some));
}
