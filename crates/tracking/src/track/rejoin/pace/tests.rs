use types::track::Track;

use super::RejoinPace;
use crate::error::Error;
use crate::fake::line;
use crate::setting::Setting;
use crate::track::rejoin::Role;

fn track(start_frame: usize, frames: usize, start: (f64, f64), step: (f64, f64)) -> Track {
    Track::new(0, line(start_frame, frames, start, step)).unwrap()
}

fn kept() -> Track {
    track(0, 10, (100.0, 100.0), (10.0, 0.0))
}

fn keeps(ended: (&Track, Role), started: (&Track, Role)) -> bool {
    RejoinPace::default().keeps(ended, started)
}

#[test]
fn a_piece_flying_on_at_the_pace_of_the_track_keeps_it() {
    let piece = track(20, 3, (300.0, 100.0), (10.0, 0.0));
    assert!(keeps((&kept(), Role::Kept), (&piece, Role::Piece)));
}

#[test]
fn a_piece_that_slows_down_suddenly_does_not_keep_the_pace() {
    let piece = track(20, 3, (300.0, 100.0), (4.0, 0.0));
    assert!(!keeps((&kept(), Role::Kept), (&piece, Role::Piece)));
    let steady = track(20, 3, (300.0, 100.0), (4.5, 0.0));
    assert!(keeps((&kept(), Role::Kept), (&steady, Role::Piece)));
}

#[test]
fn a_piece_found_where_the_track_vanished_long_after_is_a_junction_too_slow() {
    let lingering = track(20, 3, (195.0, 100.0), (10.0, 0.0));
    assert!(!keeps((&kept(), Role::Kept), (&lingering, Role::Piece)));
}

#[test]
fn a_one_point_piece_is_judged_by_its_junction_alone() {
    let ahead = track(20, 1, (300.0, 100.0), (0.0, 0.0));
    let behind = track(20, 1, (210.0, 100.0), (0.0, 0.0));
    assert!(keeps((&kept(), Role::Kept), (&ahead, Role::Piece)));
    assert!(!keeps((&kept(), Role::Kept), (&behind, Role::Piece)));
}

#[test]
fn a_piece_before_a_track_is_judged_against_the_start_of_that_track() {
    let piece = track(0, 3, (190.0, 100.0), (10.0, 0.0));
    let started = track(14, 10, (240.0, 100.0), (10.0, 0.0));
    let crawling = track(0, 3, (208.0, 100.0), (1.0, 0.0));
    assert!(keeps((&piece, Role::Piece), (&started, Role::Kept)));
    assert!(!keeps((&crawling, Role::Piece), (&started, Role::Kept)));
}

#[test]
fn a_turn_at_full_speed_keeps_the_pace_because_only_speed_is_judged() {
    let turning = track(20, 3, (300.0, 100.0), (0.0, 10.0));
    assert!(keeps((&kept(), Role::Kept), (&turning, Role::Piece)));
}

#[test]
fn paces_must_be_finite_and_not_negative() {
    assert_eq!(
        RejoinPace::new(-0.1, 0.45),
        Err(Error::Setting {
            setting: Setting::RejoinJunctionPace,
            value: -0.1
        })
    );
    assert!(matches!(
        RejoinPace::new(0.25, f64::NAN),
        Err(Error::Setting {
            setting: Setting::RejoinPiecePace,
            ..
        })
    ));
    assert_eq!(RejoinPace::new(0.0, 0.0).map(RejoinPace::junction), Ok(0.0));
}

#[test]
fn between_two_kept_tracks_only_the_junction_is_judged_against_both() {
    let started = track(22, 10, (320.0, 100.0), (10.0, 0.0));
    let slow_start = track(22, 10, (320.0, 100.0), (1.0, 0.0));
    let lingering = track(22, 10, (200.0, 100.0), (10.0, 0.0));
    assert!(keeps((&kept(), Role::Kept), (&started, Role::Kept)));
    assert!(keeps((&kept(), Role::Kept), (&slow_start, Role::Kept)));
    assert!(!keeps((&kept(), Role::Kept), (&lingering, Role::Kept)));
}

#[test]
fn a_junction_at_exactly_a_quarter_of_the_track_speed_keeps_the_pace() {
    let exact = track(21, 3, (220.0, 100.0), (10.0, 0.0));
    let short = track(21, 3, (219.9, 100.0), (10.0, 0.0));
    assert!(keeps((&kept(), Role::Kept), (&exact, Role::Piece)));
    assert!(!keeps((&kept(), Role::Kept), (&short, Role::Piece)));
}

#[test]
fn a_junction_between_kept_tracks_must_keep_the_pace_of_the_faster_of_the_two() {
    let slow_end = track(0, 10, (100.0, 100.0), (1.0, 0.0));
    let fast_start = track(22, 10, (148.0, 100.0), (20.0, 0.0));
    let steady_start = track(22, 10, (148.0, 100.0), (10.0, 0.0));
    assert!(!keeps((&slow_end, Role::Kept), (&fast_start, Role::Kept)));
    assert!(keeps((&slow_end, Role::Kept), (&steady_start, Role::Kept)));
}
