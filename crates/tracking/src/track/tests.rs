use std::f64::consts::PI;
use std::num::NonZeroUsize;

use proptest::prelude::*;
use types::detection::Detection;
use types::track::Track;

use super::{RejoinConfig, TrackConfig, track_detections};
use crate::error::Error;
use crate::fake::{detection, excluded, frames_of, line, size, span, weak};
use crate::setting::Setting;

const SCALE: f64 = 3.0;

fn count(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}

fn config(max_jump: f64, max_gap: usize, min_hits: usize, min_travel: f64, twin: Option<f64>) -> TrackConfig {
    TrackConfig::new(max_jump, count(max_gap), count(min_hits), min_travel, twin, 0.8, None).unwrap()
}

fn scaled(start_frame: usize, frames: usize, start: (f64, f64), step: (f64, f64)) -> Vec<Detection> {
    line(
        start_frame,
        frames,
        (start.0 * SCALE, start.1 * SCALE),
        (step.0 * SCALE, step.1 * SCALE),
    )
}

fn tracked(detections: &[Detection], config: TrackConfig) -> Vec<Track> {
    let frame_count = detections.iter().map(|point| point.frame.get() + 1).max().unwrap_or(0);
    track_detections(
        &excluded(frames_of(detections, frame_count), Vec::new()),
        size(),
        config,
    )
    .unwrap()
}

fn zigzag(amplitude: f64) -> Vec<Detection> {
    (0..12_u32)
        .map(|step| {
            let side = if step % 2 == 1 { amplitude } else { -amplitude };
            detection(step as usize, 20.0 * f64::from(step), 100.0 + side)
        })
        .collect()
}

#[test]
fn a_straight_flight_gives_one_track() {
    let tracks = tracked(&scaled(0, 10, (10.0, 50.0), (5.0, 0.0)), TrackConfig::default());
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].points().len(), 10);
}

#[test]
fn two_crossing_flights_stay_two_tracks() {
    let mut both = scaled(0, 12, (10.0, 10.0), (6.0, 6.0));
    both.extend(scaled(0, 12, (10.0, 76.0), (6.0, -6.0)));
    let tracks = tracked(&both, TrackConfig::default());
    let ends: Vec<(f64, f64)> = tracks
        .iter()
        .map(|track| (track.first().y.round(), track.last().y.round()))
        .collect();
    assert_eq!(ends.len(), 2);
    assert!(ends.contains(&(30.0, 228.0)) && ends.contains(&(228.0, 30.0)));
}

#[test]
fn a_gap_shorter_than_max_gap_is_bridged() {
    let flight: Vec<Detection> = scaled(0, 14, (10.0, 50.0), (4.0, 0.0))
        .into_iter()
        .filter(|point| !(6..=8).contains(&point.frame.get()))
        .collect();
    assert_eq!(tracked(&flight, TrackConfig::default()).len(), 1);
}

#[test]
fn a_gap_longer_than_max_gap_splits_the_track() {
    let flight: Vec<Detection> = scaled(0, 20, (10.0, 50.0), (4.0, 0.0))
        .into_iter()
        .filter(|point| !(6..=13).contains(&point.frame.get()))
        .collect();
    assert_eq!(tracked(&flight, config(10.0 * SCALE, 6, 6, 45.0, Some(36.0))).len(), 2);
}

fn hidden_behind_vegetation() -> Vec<Detection> {
    line(0, 40, (200.0, 300.0), (10.0, 0.0))
        .into_iter()
        .filter(|point| !(12..24).contains(&point.frame.get()))
        .collect()
}

#[test]
fn a_flight_hidden_longer_than_max_gap_is_one_track_only_with_the_rejoin() {
    let without = TrackConfig::new(120.0, count(6), count(6), 45.0, Some(36.0), 0.8, None).unwrap();
    assert_eq!(tracked(&hidden_behind_vegetation(), without).len(), 2);
    let tracks = tracked(&hidden_behind_vegetation(), TrackConfig::default());
    assert_eq!(tracks.len(), 1);
    assert_eq!((tracks[0].id(), tracks[0].points().len()), (1, 28));
    assert_eq!(tracks[0].parts().count(), 2);
}

#[test]
fn the_rejoin_only_takes_holes_longer_than_the_max_gap_it_is_given() {
    let short_hole: Vec<Detection> = line(0, 40, (200.0, 300.0), (10.0, 0.0))
        .into_iter()
        .filter(|point| !(12..17).contains(&point.frame.get()))
        .collect();
    let rejoin = Some(RejoinConfig::new(count(19), 60.0).unwrap());
    let config = TrackConfig::new(120.0, count(3), count(6), 45.0, Some(36.0), 0.8, rejoin).unwrap();
    assert_eq!(tracked(&short_hole, config.without_rejoin()).len(), 2);
    assert_eq!(tracked(&short_hole, config).len(), 1);
}

#[test]
fn a_rejoin_that_could_never_reach_past_max_gap_is_refused() {
    let rejoin = Some(RejoinConfig::new(count(6), 60.0).unwrap());
    match TrackConfig::new(120.0, count(6), count(6), 45.0, Some(36.0), 0.8, rejoin) {
        Err(Error::Setting { setting, value }) => assert_eq!((setting, value), (Setting::RejoinHidden, 6.0)),
        other => panic!("accepted {other:?}"),
    }
}

#[test]
fn a_rejoined_flight_is_numbered_by_its_first_part() {
    let mut both = hidden_behind_vegetation();
    both.extend(line(5, 25, (200.0, 700.0), (10.0, 0.0)));
    let tracks = tracked(&both, TrackConfig::default());
    let starts: Vec<(usize, usize, usize)> = tracks
        .iter()
        .map(|track| (track.id(), track.first().frame.get(), track.points().len()))
        .collect();
    assert_eq!(starts, [(1, 0, 28), (2, 5, 25)]);
}

#[test]
fn the_spans_ignored_by_the_exclusion_keep_a_hidden_flight_in_two_tracks() {
    let detections = frames_of(&hidden_behind_vegetation(), 40);
    let tracks = track_detections(
        &excluded(detections, vec![span(15, 16)]),
        size(),
        TrackConfig::default(),
    )
    .unwrap();
    assert_eq!(tracks.len(), 2);
}

#[test]
fn a_track_survives_exactly_max_gap_empty_frames_but_not_one_more() {
    for (missing, expected) in [(6, 1), (7, 2)] {
        let flight: Vec<Detection> = scaled(0, 30, (10.0, 50.0), (4.0, 0.0))
            .into_iter()
            .filter(|point| !(10..10 + missing).contains(&point.frame.get()))
            .collect();
        assert_eq!(
            tracked(&flight, config(120.0, 6, 6, 45.0, None)).len(),
            expected,
            "{missing} missing"
        );
    }
}

#[test]
fn tracks_below_min_hits_are_dropped_and_exactly_min_hits_is_kept() {
    let short = scaled(0, 4, (10.0, 50.0), (5.0, 0.0));
    assert_eq!(tracked(&short, config(120.0, 6, 5, 0.0, Some(36.0))), []);
    assert_eq!(tracked(&short, config(120.0, 6, 4, 0.0, Some(36.0))).len(), 1);
}

#[test]
fn a_flickering_spot_that_does_not_travel_is_dropped() {
    let flicker = scaled(0, 20, (50.0, 50.0), (0.2, 0.0));
    assert_eq!(tracked(&flicker, config(120.0, 6, 6, 15.0 * SCALE, Some(36.0))), []);
}

#[test]
fn a_chord_equal_to_min_travel_is_kept() {
    let flight = [detection(0, 0.0, 0.0), detection(1, 3.0, 4.0)];
    assert_eq!(tracked(&flight, config(120.0, 6, 1, 5.0, None)).len(), 1);
    assert_eq!(tracked(&flight, config(120.0, 6, 1, 5.001, None)), []);
}

#[test]
fn tracks_are_numbered_in_chronological_order() {
    let mut both = scaled(30, 8, (10.0, 20.0), (5.0, 0.0));
    both.extend(scaled(0, 8, (10.0, 90.0), (5.0, 0.0)));
    let tracks = tracked(&both, TrackConfig::default());
    let numbers: Vec<(usize, usize)> = tracks
        .iter()
        .map(|track| (track.id(), track.first().frame.get()))
        .collect();
    assert_eq!(numbers, [(1, 0), (2, 30)]);
}

#[test]
fn an_established_track_keeps_its_detection_over_a_closer_newborn_track() {
    let mut flight = scaled(0, 6, (10.0, 50.0), (8.0, 0.0));
    flight.extend(scaled(6, 4, (54.0, 50.0), (4.0, 0.0)));
    flight.push(detection(5, 53.0 * SCALE, 51.0 * SCALE));
    let tracks = tracked(&flight, config(120.0, 6, 6, 45.0, None));
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].points().len(), 10);
}

#[test]
fn fragments_of_one_animal_are_merged_into_one_track() {
    let mut both = scaled(0, 10, (10.0, 50.0), (6.0, 0.0));
    both.extend(scaled(0, 10, (14.0, 56.0), (6.0, 0.0)));
    let tracks = tracked(&both, config(120.0, 6, 6, 45.0, Some(12.0 * SCALE)));
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].points().len(), 10);
}

#[test]
fn without_twin_distance_fragments_stay_apart() {
    let mut both = scaled(0, 10, (10.0, 50.0), (6.0, 0.0));
    both.extend(scaled(0, 10, (14.0, 56.0), (6.0, 0.0)));
    assert_eq!(tracked(&both, config(120.0, 6, 6, 45.0, None)).len(), 2);
}

#[test]
fn a_gentle_zigzag_is_kept_but_a_wide_one_is_noise() {
    assert_eq!(tracked(&zigzag(2.0), TrackConfig::default()).len(), 1);
    assert_eq!(tracked(&zigzag(15.0), TrackConfig::default()), []);
}

#[test]
fn a_large_enough_max_median_turn_keeps_any_zigzag() {
    let lenient = TrackConfig::new(120.0, count(6), count(6), 45.0, Some(36.0), PI, None).unwrap();
    assert_eq!(tracked(&zigzag(30.0), lenient).len(), 1);
}

#[test]
fn a_track_exactly_at_the_turn_limit_is_kept() {
    let limit = Track::new(1, zigzag(15.0)).unwrap().median_turn();
    let at_limit = TrackConfig::new(120.0, count(6), count(6), 45.0, Some(36.0), limit, None).unwrap();
    assert_eq!(tracked(&zigzag(15.0), at_limit).len(), 1);
}

#[test]
fn a_video_without_detections_has_no_track() {
    assert_eq!(tracked(&[], TrackConfig::default()), []);
}

#[test]
fn the_default_settings_are_those_of_the_reference() {
    let expected = TrackConfig::new(
        120.0,
        count(6),
        count(6),
        45.0,
        Some(36.0),
        0.8,
        Some(RejoinConfig::new(count(19), 60.0).unwrap()),
    )
    .unwrap();
    assert_eq!(TrackConfig::default(), expected);
}

#[test]
fn each_setting_is_read_back_from_its_own_argument() {
    let rejoin = RejoinConfig::new(count(7), 8.0).unwrap();
    let settings = TrackConfig::new(1.0, count(2), count(3), 4.0, Some(5.0), 0.6, Some(rejoin)).unwrap();
    assert_eq!(settings.max_jump(), 1.0);
    assert_eq!((settings.max_gap().get(), settings.min_hits().get()), (2, 3));
    assert_eq!((settings.min_travel(), settings.twin_distance()), (4.0, Some(5.0)));
    assert_eq!(settings.max_median_turn(), 0.6);
    assert_eq!(settings.rejoin(), Some(rejoin));
}

#[test]
fn settings_out_of_bounds_not_a_number_or_infinite_are_refused() {
    let refused = |max_jump, min_travel, twin, turn| match TrackConfig::new(
        max_jump,
        count(6),
        count(6),
        min_travel,
        twin,
        turn,
        None,
    ) {
        Err(Error::Setting { setting, .. }) => setting,
        other => panic!("accepted {other:?}"),
    };
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(refused(bad, 45.0, Some(36.0), 0.8), Setting::MaxJump);
        assert_eq!(refused(120.0, 45.0, Some(bad), 0.8), Setting::TwinDistance);
        assert_eq!(refused(120.0, 45.0, Some(36.0), bad), Setting::MaxMedianTurn);
    }
    for bad in [-1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(refused(120.0, bad, Some(36.0), 0.8), Setting::MinTravel);
    }
}

#[test]
fn a_refused_setting_is_named_by_its_params_json_key() {
    let error = TrackConfig::new(0.0, count(6), count(6), 45.0, None, 0.8, None).unwrap_err();
    assert_eq!(error.to_string(), "max_jump is out of bounds: 0");
}

#[test]
fn a_zero_min_travel_is_accepted() {
    assert!(TrackConfig::new(120.0, count(6), count(6), 0.0, None, 0.8, None).is_ok());
}

#[test]
fn the_largest_max_gap_neither_overflows_nor_cuts_the_track() {
    let flight = scaled(0, 10, (10.0, 50.0), (5.0, 0.0));
    let patient = TrackConfig::new(120.0, NonZeroUsize::MAX, count(6), 45.0, Some(36.0), 0.8, None).unwrap();
    assert_eq!(tracked(&flight, patient).len(), 1);
}

fn scattered() -> impl Strategy<Value = Vec<Detection>> {
    prop::collection::vec((0_usize..40, 0.0..600.0_f64, 0.0..600.0_f64), 0..80)
        .prop_map(|points| points.into_iter().map(|(frame, x, y)| detection(frame, x, y)).collect())
}

proptest! {
    #[test]
    fn tracks_are_numbered_one_by_one_in_order_of_start_then_x(points in scattered()) {
        let lenient = TrackConfig::new(120.0, count(6), count(2), 0.0, Some(36.0), PI, Some(RejoinConfig::default())).unwrap();
        let tracks = tracked(&points, lenient);
        for (index, track) in tracks.iter().enumerate() {
            prop_assert_eq!(track.id(), index + 1);
        }
        for pair in tracks.windows(2) {
            let (a, b) = (pair[0].first(), pair[1].first());
            prop_assert!(a.frame < b.frame || (a.frame == b.frame && a.x <= b.x));
        }
    }
}

#[test]
fn a_flight_seen_only_through_weak_detections_opens_no_track() {
    let faint: Vec<Detection> = line(0, 12, (10.0, 50.0), (20.0, 0.0))
        .iter()
        .map(|point| weak(point.frame.get(), point.x, point.y))
        .collect();
    assert_eq!(tracked(&faint, TrackConfig::default()), []);
}

#[test]
fn a_weak_detection_fills_the_frame_a_strong_flight_misses() {
    let mut flight = line(0, 12, (10.0, 50.0), (20.0, 0.0));
    flight[6] = weak(6, flight[6].x, flight[6].y);
    let tracks = tracked(&flight, TrackConfig::default());
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].points().len(), 12);
}

#[test]
fn a_weak_detection_between_two_close_flights_does_not_join_them() {
    let mut both = [
        line(0, 12, (10.0, 100.0), (20.0, 0.0)),
        line(0, 12, (10.0, 140.0), (20.0, 0.0)),
    ]
    .concat();
    both.push(weak(6, 130.0, 120.0));
    assert_eq!(tracked(&both, TrackConfig::default()).len(), 2);
}
