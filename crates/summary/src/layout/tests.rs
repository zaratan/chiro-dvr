use std::num::NonZeroU32;

use draw::style::style_for;
use draw::text::Typeface;
use types::frame::Pts;
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;

use files::names::Part;

use super::{MAX_TRACKS_PER_IMAGE, Summary, layout};
use crate::header::HeaderLine;
use crate::placement::Frame;
use crate::testing::track;

#[test]
fn the_legend_time_of_a_track_comes_from_the_pts_of_its_first_frame() {
    let rate = FrameRate::new(Rational::new(10, 1).unwrap()).unwrap();
    let late = |frame: i64| Some(Pts::new(if frame < 5 { frame } else { frame + 600 }));
    let timeline = Timeline::new((0..20).map(late).collect(), Rational::new(1, 10).unwrap(), rate).unwrap();
    let tracks = [track(1, 10, &[[10.0, 10.0], [60.0, 10.0], [110.0, 10.0]])];
    let typeface = Typeface::roboto().unwrap();
    let style = style_for(NonZeroU32::new(320).unwrap(), "1", &typeface);
    let summary = Summary {
        name: "v",
        tracks: &tracks,
        timeline: &timeline,
        ignored: &[],
        damaged: &[],
    };
    let frame = Frame {
        width: 320.0,
        height: 240.0,
    };
    let pages = layout(&summary, frame, &style);
    assert_eq!(pages[0].marks[0].start, "1:01");
}

fn crowded(count: usize) -> Vec<types::track::Track> {
    (0..count)
        .map(|index| {
            let y = 20.0 + 4.0 * index as f64;
            track(index + 1, 10 * index, &[[20.0, y], [160.0, y], [300.0, y]])
        })
        .collect()
}

fn pages_of<'a>(tracks: &'a [types::track::Track], timeline: &'a Timeline) -> Vec<super::PageLayout<'a>> {
    let typeface = Typeface::roboto().unwrap();
    let style = style_for(NonZeroU32::new(320).unwrap(), "99", &typeface);
    let summary = Summary {
        name: "v",
        tracks,
        timeline,
        ignored: &[],
        damaged: &[],
    };
    let frame = Frame {
        width: 320.0,
        height: 240.0,
    };
    layout(&summary, frame, &style)
}

fn part(number: usize, count: usize) -> Part {
    Part { number, count }
}

#[test]
fn a_period_of_up_to_twenty_tracks_stays_one_image_without_a_part() {
    let tracks = crowded(MAX_TRACKS_PER_IMAGE);
    let timeline = crate::testing::timeline(3000);
    let pages = pages_of(&tracks, &timeline);
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].part, None);
    assert_eq!(pages[0].marks.len(), MAX_TRACKS_PER_IMAGE);
    assert_eq!(pages[0].other_starts.len(), 0);
    assert_eq!(pages[0].header[1], HeaderLine::Tracks(MAX_TRACKS_PER_IMAGE));
}

#[test]
fn a_crowded_period_is_cut_into_equal_consecutive_parts_of_at_most_twenty_numbered_over_the_video() {
    let tracks = crowded(45);
    let timeline = crate::testing::timeline(3000);
    let pages = pages_of(&tracks, &timeline);
    let parts: Vec<_> = pages.iter().map(|page| page.part).collect();
    assert_eq!(parts, [Some(part(1, 3)), Some(part(2, 3)), Some(part(3, 3))]);
    let ids: Vec<Vec<usize>> = pages
        .iter()
        .map(|page| page.marks.iter().map(|mark| mark.track.id()).collect())
        .collect();
    assert_eq!(
        ids,
        [(1..=15).collect::<Vec<_>>(), (16..=30).collect(), (31..=45).collect()]
    );
    assert_eq!(pages[1].header[1], HeaderLine::Part(part(2, 3)));
    assert_eq!(pages[1].header[2], HeaderLine::Tracks(15));
    let bits = |window: [f64; 2]| window.map(f64::to_bits);
    assert!(pages.iter().all(|page| bits(page.window) == bits(pages[0].window)));
}

#[test]
fn the_other_parts_lend_the_strip_their_start_times_and_only_theirs() {
    let tracks = crowded(45);
    let timeline = crate::testing::timeline(3000);
    let pages = pages_of(&tracks, &timeline);
    let expected: Vec<f64> = (0..15).chain(30..45).map(f64::from).collect();
    assert_eq!(pages[1].other_starts, expected);
}

#[test]
fn each_part_spreads_the_palette_over_its_own_tracks_only() {
    let tracks = crowded(45);
    let timeline = crate::testing::timeline(3000);
    let pages = pages_of(&tracks, &timeline);
    let near =
        f64::from(style_for(NonZeroU32::new(320).unwrap(), "99", &Typeface::roboto().unwrap()).neighbor_distance);
    for page in &pages {
        let paths: Vec<_> = page.marks.iter().map(|mark| mark.path.clone()).collect();
        let own = crate::colors::assign_colors(&paths, draw::color::PALETTE.len(), near);
        let colors: Vec<usize> = page.marks.iter().map(|mark| mark.color).collect();
        assert_eq!(colors, own);
    }
    assert_eq!(pages[1].marks[0].color, 0);
}

#[test]
fn twenty_one_tracks_make_two_parts_of_eleven_and_ten_rather_than_twenty_and_one() {
    let tracks = crowded(21);
    let timeline = crate::testing::timeline(3000);
    let sizes: Vec<usize> = pages_of(&tracks, &timeline)
        .iter()
        .map(|page| page.marks.len())
        .collect();
    assert_eq!(sizes, [11, 10]);
}

#[test]
fn a_part_header_runs_from_its_first_start_to_its_latest_end_rounded_up() {
    let mut tracks = crowded(45);
    let long: Vec<[f64; 2]> = (0..250).map(|step| [20.0 + f64::from(step), 100.0]).collect();
    tracks[20] = track(21, 200, &long);
    let timeline = crate::testing::timeline(3000);
    let pages = pages_of(&tracks, &timeline);
    assert_eq!(
        pages[1].header[3],
        HeaderLine::Window {
            start: "0:15".to_owned(),
            end: "0:45".to_owned()
        }
    );
}

#[test]
fn only_the_crowded_period_of_a_long_video_is_cut_and_its_strip_keeps_to_that_period() {
    let mut tracks = crowded(3);
    tracks.extend((0..25).map(|index| {
        let y = 20.0 + 6.0 * index as f64;
        track(4 + index, 6100 + 10 * index, &[[20.0, y], [160.0, y], [300.0, y]])
    }));
    let timeline = crate::testing::timeline(13_000);
    let pages = pages_of(&tracks, &timeline);
    let parts: Vec<_> = pages.iter().map(|page| (page.minutes.clone(), page.part)).collect();
    assert_eq!(
        parts,
        [
            (Some(0..10), None),
            (Some(10..20), Some(part(1, 2))),
            (Some(10..20), Some(part(2, 2))),
            (Some(20..22), None),
        ]
    );
    assert!(
        pages[2]
            .other_starts
            .iter()
            .all(|&seconds| (600.0..1200.0).contains(&seconds))
    );
    assert_eq!(pages[2].other_starts.len(), 13);
}
