use types::detection::Detection;
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::track::Track;

use super::{DiscrepancyKind, match_tracks};
use crate::location::committed;
use crate::recording::Recording;

fn track(id: usize, start: usize, points: usize) -> Track {
    let points = (start..start + points)
        .map(|frame| Detection {
            frame: FrameIndex::new(frame),
            x: frame as f64,
            y: 10.0,
            left: frame as f64 - 1.0,
            top: 9.0,
            width: 3.0,
            height: 3.0,
            area: 9.0,
            amplitude: 20.0,
        })
        .collect();
    Track::new(id, points).unwrap()
}

fn rate() -> FrameRate {
    FrameRate::new(Rational::new(10_000, 333).unwrap()).unwrap()
}

#[test]
fn the_reference_tracks_match_themselves_completely() {
    let full = Recording::read(&committed("video_092_original")).unwrap();
    let matching = match_tracks(&full.tracks, &full.tracks);
    assert_eq!((matching.pairs.len(), matching.agreeing()), (14, 14));
    let discrepancies = matching.discrepancies("video_092_original.mp4", full.timeline().unwrap().rate());
    assert_eq!(discrepancies, []);
}

#[test]
fn a_start_five_frames_off_pairs_but_six_does_not() {
    let reference = [track(1, 100, 20)];
    assert_eq!(match_tracks(&reference, &[track(1, 105, 20)]).pairs.len(), 1);
    let apart = [track(1, 106, 20)];
    let matching = match_tracks(&reference, &apart);
    assert_eq!(matching.pairs, []);
    let kinds: Vec<_> = matching
        .discrepancies("v.mp4", rate())
        .into_iter()
        .map(|discrepancy| (discrepancy.frame.get(), discrepancy.kind))
        .collect();
    assert_eq!(
        kinds,
        [
            (100, DiscrepancyKind::Missed { reference_id: 1 }),
            (106, DiscrepancyKind::Extra { candidate_id: 1 })
        ]
    );
}

#[test]
fn three_points_more_agree_but_four_is_a_discrepancy() {
    let reference = [track(1, 100, 20), track(2, 500, 20)];
    let candidate = [track(1, 100, 23), track(2, 500, 24)];
    let matching = match_tracks(&reference, &candidate);
    assert_eq!((matching.pairs.len(), matching.agreeing()), (2, 1));
    let discrepancies = matching.discrepancies("v.mp4", rate());
    assert_eq!(discrepancies.len(), 1);
    assert_eq!(
        discrepancies[0].kind,
        DiscrepancyKind::Points {
            reference_id: 2,
            candidate_id: 2,
            reference_points: 20,
            candidate_points: 24
        }
    );
}

#[test]
fn the_closest_start_wins_whatever_the_order_of_the_tracks() {
    let reference = [track(1, 100, 20)];
    let candidate = [track(1, 103, 20), track(2, 101, 20)];
    let matching = match_tracks(&reference, &candidate);
    assert_eq!(matching.pairs[0].1.id(), 2);
    assert_eq!(matching.candidate_only[0].id(), 1);
}

#[test]
fn at_equal_starts_the_closest_number_of_points_wins() {
    let reference = [track(1, 100, 20)];
    let candidate = [track(1, 100, 30), track(2, 100, 21)];
    assert_eq!(match_tracks(&reference, &candidate).pairs[0].1.id(), 2);
}

#[test]
fn discrepancies_name_the_video_and_the_time_in_frame_order() {
    let reference = [track(1, 999, 20), track(2, 30, 20)];
    let discrepancies = match_tracks(&reference, &[]).discrepancies("video_125.mp4", rate());
    let found: Vec<_> = discrepancies
        .iter()
        .map(|discrepancy| (discrepancy.video.as_str(), discrepancy.frame.get()))
        .collect();
    assert_eq!(found, [("video_125.mp4", 30), ("video_125.mp4", 999)]);
    assert!((discrepancies[1].seconds - 33.2667).abs() < 1e-9);
}

#[test]
fn pairing_takes_the_closest_starts_first_even_when_it_leaves_a_track_alone() {
    let reference = [track(1, 100, 20), track(2, 110, 20)];
    let candidate = [track(1, 105, 20), track(2, 95, 20)];
    let matching = match_tracks(&reference, &candidate);
    assert_eq!(matching.pairs.len(), 1);
    assert_eq!((matching.pairs[0].0.id(), matching.pairs[0].1.id()), (1, 1));
}
