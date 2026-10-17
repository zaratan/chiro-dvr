use std::num::NonZeroUsize;

use proptest::prelude::*;
use types::detection::Detection;
use types::span::Span;
use types::track::Track;

use super::{Hiding, RejoinConfig, rejoin};
use crate::error::Error;
use crate::fake::{line, size, span};
use crate::setting::Setting;

const MAX_GAP: usize = 6;

fn config(max_hidden: usize, max_distance: f64) -> RejoinConfig {
    RejoinConfig::new(NonZeroUsize::new(max_hidden).unwrap(), max_distance).unwrap()
}

fn hiding(ignored: &[Span]) -> Hiding<'_> {
    Hiding {
        longer_than: MAX_GAP,
        ignored,
        size: size(),
    }
}

fn rejoined(tracks: &[Track], max_hidden: usize, max_distance: f64) -> Vec<Track> {
    rejoin(tracks, hiding(&[]), config(max_hidden, max_distance)).unwrap()
}

fn track(points: Vec<Detection>) -> Track {
    Track::new(0, points).unwrap()
}

fn before_the_vegetation() -> Track {
    track(line(0, 10, (100.0, 100.0), (10.0, 0.0)))
}

fn after_the_vegetation(start_frame: usize, offset: (f64, f64)) -> Track {
    let steps_since_seen = (start_frame - 9) as f64;
    track(line(
        start_frame,
        10,
        (190.0 + 10.0 * steps_since_seen + offset.0, 100.0 + offset.1),
        (10.0, 0.0),
    ))
}

fn lengths(tracks: &[Track]) -> Vec<usize> {
    let mut lengths: Vec<usize> = tracks.iter().map(|track| track.points().len()).collect();
    lengths.sort_unstable();
    lengths
}

#[test]
fn a_flight_cut_by_a_hiding_place_becomes_one_track_without_invented_points() {
    let joined = rejoined(
        &[before_the_vegetation(), after_the_vegetation(22, (0.0, 0.0))],
        19,
        60.0,
    );
    assert_eq!(joined.len(), 1);
    let frames: Vec<usize> = joined[0].points().iter().map(|point| point.frame.get()).collect();
    assert_eq!(frames, [(0..10).collect::<Vec<_>>(), (22..32).collect()].concat());
}

#[test]
fn a_flight_hidden_exactly_the_frames_allowed_is_rejoined_but_not_one_frame_more() {
    for (start_frame, expected) in [(22, 1), (23, 2)] {
        let parts = [before_the_vegetation(), after_the_vegetation(start_frame, (0.0, 0.0))];
        assert_eq!(rejoined(&parts, 12, 60.0).len(), expected, "start {start_frame}");
    }
}

#[test]
fn a_hole_no_longer_than_max_gap_is_left_to_the_association_that_already_refused_it() {
    for (start_frame, expected) in [(10, 2), (16, 2), (17, 1)] {
        let parts = [before_the_vegetation(), after_the_vegetation(start_frame, (0.0, 0.0))];
        assert_eq!(rejoined(&parts, 19, 60.0).len(), expected, "start {start_frame}");
    }
}

#[test]
fn a_start_exactly_at_the_rejoin_distance_from_the_meeting_point_is_rejoined_but_not_farther() {
    for (offset, expected) in [(60.0, 1), (60.5, 2)] {
        let parts = [before_the_vegetation(), after_the_vegetation(22, (0.0, offset))];
        assert_eq!(rejoined(&parts, 19, 60.0).len(), expected, "{offset}");
    }
}

#[test]
fn a_turn_while_hidden_is_rejoined_because_both_ends_point_to_where_they_meet() {
    let diving_left = track(line(0, 10, (500.0, 400.0), (-10.0, 10.0)));
    let diving_right = track(line(21, 10, (410.0, 610.0), (10.0, 10.0)));
    assert_eq!(rejoined(&[diving_left, diving_right], 19, 60.0).len(), 1);
}

#[test]
fn the_heading_comes_from_the_last_points_before_hiding_not_from_the_whole_flight() {
    let mut turning_down = line(0, 20, (300.0, 300.0), (10.0, 0.0));
    turning_down.extend(line(20, 5, (490.0, 310.0), (0.0, 10.0)));
    let going_on_down = track(line(37, 10, (490.0, 480.0), (0.0, 10.0)));
    assert_eq!(rejoined(&[track(turning_down), going_on_down], 19, 60.0).len(), 1);
}

#[test]
fn a_hole_touched_by_an_ignored_span_is_never_rejoined() {
    let cases = [
        (span(15, 16), 2),
        (span(10, 21), 2),
        (span(10, 10), 2),
        (span(21, 21), 2),
        (span(0, 9), 1),
        (span(22, 30), 1),
        (span(40, 45), 1),
    ];
    for (ignored, expected) in cases {
        let parts = [before_the_vegetation(), after_the_vegetation(22, (0.0, 0.0))];
        let joined = rejoin(&parts, hiding(&[ignored]), config(19, 60.0)).unwrap();
        assert_eq!(joined.len(), expected, "{ignored:?}");
    }
}

#[test]
fn a_flight_that_leaves_the_image_and_comes_back_is_never_rejoined() {
    for (end_x, expected) in [(1411.0, 2), (1410.0, 1)] {
        let leaving = track(line(0, 10, (end_x - 90.0, 500.0), (10.0, 0.0)));
        let coming_back = track(line(22, 10, (end_x, 500.0), (-10.0, 0.0)));
        assert_eq!(rejoined(&[leaving, coming_back], 19, 60.0).len(), expected, "{end_x}");
    }
}

#[test]
fn a_flight_that_leaves_by_the_top_or_the_bottom_and_comes_back_is_never_rejoined() {
    for (end_y, expected) in [(29.0, 2), (30.0, 1), (1051.0, 2), (1050.0, 1)] {
        let leaving = track(line(0, 10, (500.0, end_y), (10.0, 0.0)));
        let coming_back = track(line(22, 10, (720.0, end_y), (10.0, 0.0)));
        assert_eq!(rejoined(&[leaving, coming_back], 19, 60.0).len(), expected, "{end_y}");
    }
}

#[test]
fn a_track_that_enters_from_the_edge_is_never_the_continuation_of_one_inside() {
    let inside = track(line(0, 10, (1200.0, 500.0), (10.0, 0.0)));
    let entering = track(line(22, 10, (1420.0, 500.0), (10.0, 0.0)));
    assert_eq!(rejoined(&[inside, entering], 19, 60.0).len(), 2);
}

#[test]
fn an_end_continues_into_the_closest_start_only_and_leaves_the_other_animal_apart() {
    let closest = after_the_vegetation(22, (0.0, 5.0));
    let other_animal = after_the_vegetation(22, (0.0, 30.0));
    let joined = rejoined(&[before_the_vegetation(), other_animal, closest], 19, 60.0);
    assert_eq!(lengths(&joined), [10, 20]);
    let whole = joined.iter().find(|track| track.points().len() == 20).unwrap();
    assert_eq!(whole.last().y, 105.0);
}

#[test]
fn a_start_continues_the_closest_end_only() {
    let other_animal = track(line(0, 10, (100.0, 140.0), (10.0, 0.0)));
    let joined = rejoined(
        &[
            other_animal,
            before_the_vegetation(),
            after_the_vegetation(22, (0.0, 0.0)),
        ],
        19,
        60.0,
    );
    assert_eq!(lengths(&joined), [10, 20]);
    let whole = joined.iter().find(|track| track.points().len() == 20).unwrap();
    assert_eq!(whole.first().y, 100.0);
}

#[test]
fn two_animals_hidden_at_the_same_time_each_get_their_own_continuation() {
    let upper = track(line(0, 10, (100.0, 300.0), (10.0, 0.0)));
    let lower = track(line(0, 10, (100.0, 340.0), (10.0, 0.0)));
    let upper_again = track(line(22, 10, (320.0, 300.0), (10.0, 0.0)));
    let lower_again = track(line(22, 10, (320.0, 340.0), (10.0, 0.0)));
    let joined = rejoined(&[lower_again, upper, upper_again, lower], 19, 60.0);
    assert_eq!(lengths(&joined), [20, 20]);
    for whole in &joined {
        assert!(
            whole
                .points()
                .iter()
                .all(|point| point.y.total_cmp(&whole.first().y).is_eq())
        );
    }
}

#[test]
fn a_flight_hidden_twice_becomes_one_track() {
    let first_hiding = after_the_vegetation(22, (0.0, 0.0));
    let second_hiding = track(line(44, 10, (540.0, 100.0), (10.0, 0.0)));
    let joined = rejoined(&[second_hiding, before_the_vegetation(), first_hiding], 19, 60.0);
    assert_eq!(lengths(&joined), [30]);
    let starts: Vec<usize> = joined[0].parts().map(|part| part.points()[0].frame.get()).collect();
    assert_eq!(starts, [0, 22, 44]);
}

#[test]
fn a_track_left_alone_by_the_rejoin_stays_a_single_part() {
    let joined = rejoined(&[before_the_vegetation()], 19, 60.0);
    assert_eq!(joined[0].parts().count(), 1);
}

#[test]
fn a_track_that_starts_before_the_other_ends_is_never_its_continuation() {
    let overlapping = track(line(9, 10, (190.0, 100.0), (10.0, 0.0)));
    assert_eq!(rejoined(&[before_the_vegetation(), overlapping], 19, 60.0).len(), 2);
}

#[test]
fn single_point_tracks_are_rejoined_by_their_position_alone() {
    let here = track(line(0, 1, (100.0, 100.0), (0.0, 0.0)));
    let there = track(line(12, 1, (130.0, 100.0), (0.0, 0.0)));
    assert_eq!(rejoined(&[here, there], 19, 60.0).len(), 1);
}

#[test]
fn no_track_gives_no_track() {
    assert_eq!(rejoined(&[], 19, 60.0), []);
}

#[test]
fn the_default_rejoin_is_the_one_measured_on_the_twenty_six_videos() {
    assert_eq!(RejoinConfig::default(), config(19, 60.0));
}

#[test]
fn a_rejoin_distance_out_of_bounds_not_a_number_or_infinite_is_refused() {
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        match RejoinConfig::new(NonZeroUsize::MIN, bad) {
            Err(Error::Setting { setting, .. }) => assert_eq!(setting, Setting::RejoinDistance),
            other => panic!("accepted {bad}: {other:?}"),
        }
    }
}

fn flights() -> impl Strategy<Value = Vec<Track>> {
    prop::collection::vec(
        (
            0_usize..60,
            1_usize..12,
            30.0..600.0_f64,
            30.0..600.0_f64,
            -15.0..15.0_f64,
        ),
        0..12,
    )
    .prop_map(|flights| {
        flights
            .into_iter()
            .map(|(start, frames, x, y, step)| track(line(start, frames, (x, y), (step, step / 2.0))))
            .collect()
    })
}

fn every_point(tracks: &[Track]) -> Vec<(usize, u64, u64)> {
    let mut points: Vec<(usize, u64, u64)> = tracks
        .iter()
        .flat_map(Track::points)
        .map(|point| (point.frame.get(), point.x.to_bits(), point.y.to_bits()))
        .collect();
    points.sort_unstable();
    points
}

proptest! {
    #[test]
    fn rejoining_keeps_every_detection_once_and_never_adds_a_track(tracks in flights()) {
        let joined = rejoined(&tracks, 19, 60.0);
        prop_assert!(joined.len() <= tracks.len());
        prop_assert_eq!(every_point(&joined), every_point(&tracks));
    }

}
