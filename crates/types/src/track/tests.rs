use std::f64::consts::{FRAC_PI_2, PI};

use super::{Error, Track, median};
use crate::detection::Detection;
use crate::frame::FrameIndex;

fn detection(frame: usize, x: f64, y: f64) -> Detection {
    Detection {
        frame: FrameIndex::new(frame),
        x,
        y,
        left: x - 1.0,
        top: y - 1.0,
        width: 3.0,
        height: 3.0,
        area: 9.0,
        amplitude: 80.0,
    }
}

fn track(positions: &[(f64, f64)]) -> Track {
    let points = positions
        .iter()
        .enumerate()
        .map(|(frame, &(x, y))| detection(frame, x, y))
        .collect();
    Track::new(1, points).unwrap()
}

fn zigzag(amplitude: f64, heading: f64) -> Track {
    let positions: Vec<_> = (0..12_u32)
        .map(|step| {
            let side = if step % 2 == 1 { amplitude } else { -amplitude };
            (400.0 + heading * 20.0 * f64::from(step), 100.0 + side)
        })
        .collect();
    track(&positions)
}

#[test]
fn a_track_without_points_is_refused() {
    assert_eq!(Track::new(3, vec![]), Err(Error::Empty { id: 3 }));
}

#[test]
fn a_track_going_back_in_time_or_twice_on_one_frame_is_refused() {
    let twice = Track::new(2, vec![detection(4, 0.0, 0.0), detection(4, 1.0, 0.0)]);
    assert_eq!(
        twice,
        Err(Error::Unordered {
            id: 2,
            frame: FrameIndex::new(4)
        })
    );
    let back = Track::new(2, vec![detection(5, 0.0, 0.0), detection(3, 1.0, 0.0)]).unwrap_err();
    assert_eq!(back.to_string(), "track 2 does not move forward in time at frame 3");
}

#[test]
fn a_gap_between_frames_is_allowed() {
    let gapped = Track::new(7, vec![detection(10, 0.0, 0.0), detection(14, 30.0, 40.0)]).unwrap();
    assert_eq!(gapped.id(), 7);
    assert_eq!(
        (gapped.first().frame, gapped.last().frame),
        (FrameIndex::new(10), FrameIndex::new(14))
    );
    assert_eq!(gapped.points().len(), 2);
}

#[test]
fn the_chord_joins_the_first_and_last_points_whatever_the_path() {
    let detour = track(&[(0.0, 0.0), (0.0, 40.0), (30.0, 40.0), (30.0, 0.0)]);
    assert!((detour.chord() - 30.0).abs() < 1e-12);
    assert!((detour.path_length() - 110.0).abs() < 1e-12);
}

#[test]
fn a_single_point_has_neither_chord_nor_path() {
    let single = track(&[(5.0, 5.0)]);
    assert_eq!((single.chord(), single.path_length()), (0.0, 0.0));
}

#[test]
fn a_single_point_path_is_a_positive_zero_so_it_never_prints_as_minus_zero() {
    assert!(track(&[(5.0, 5.0)]).path_length().is_sign_positive());
}

#[test]
fn renumbering_a_track_keeps_its_points() {
    let gapped = Track::new(7, vec![detection(10, 0.0, 0.0), detection(14, 30.0, 40.0)]).unwrap();
    let renumbered = gapped.clone().with_id(2);
    assert_eq!((renumbered.id(), renumbered.points()), (2, gapped.points()));
}

#[test]
fn a_track_never_rejoined_is_a_single_part_with_all_its_points() {
    let whole = track(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)]);
    let parts: Vec<&[Detection]> = whole.parts().map(|part| part.points()).collect();
    assert_eq!(parts, [whole.points()]);
}

#[test]
fn a_rejoined_track_keeps_each_part_apart_through_renumbering() {
    let before = vec![detection(0, 0.0, 0.0), detection(1, 10.0, 0.0)];
    let after = vec![detection(12, 120.0, 0.0)];
    let joined = Track::rejoined(0, vec![before.clone(), after.clone()])
        .unwrap()
        .with_id(3);
    let parts: Vec<&[Detection]> = joined.parts().map(|part| part.points()).collect();
    assert_eq!(parts, [before.as_slice(), after.as_slice()]);
    assert_eq!(joined.points(), [before, after].concat());
}

#[test]
fn a_track_rejoined_from_a_single_part_is_the_plain_track() {
    let points = vec![detection(0, 0.0, 0.0), detection(1, 10.0, 0.0)];
    assert_eq!(Track::rejoined(2, vec![points.clone()]), Track::new(2, points));
    assert_eq!(Track::rejoined(2, Vec::new()), Err(Error::Empty { id: 2 }));
}

#[test]
fn a_rejoined_track_with_an_empty_part_is_refused_rather_than_measured_as_nothing() {
    assert_eq!(
        Track::rejoined(4, vec![vec![detection(0, 0.0, 0.0)], Vec::new()]),
        Err(Error::Empty { id: 4 })
    );
}

#[test]
fn rejoined_parts_out_of_time_order_are_refused() {
    assert_eq!(
        Track::rejoined(4, vec![vec![detection(5, 0.0, 0.0)], vec![detection(2, 0.0, 0.0)]]),
        Err(Error::Unordered {
            id: 4,
            frame: FrameIndex::new(2)
        })
    );
}

#[test]
fn largest_area_and_peak_amplitude_may_come_from_different_detections() {
    let measures = [(40.0, 22.5), (12.0, 61.0), (25.0, 30.0)];
    let points = measures
        .iter()
        .enumerate()
        .map(|(frame, &(area, amplitude))| Detection {
            area,
            amplitude,
            ..detection(frame, 10.0 * frame as f64, 10.0)
        })
        .collect();
    let track = Track::new(1, points).unwrap();
    assert_eq!((track.max_area(), track.peak_amplitude()), (40.0, 61.0));
}

#[test]
fn a_curve_turning_steadily_has_a_small_median_turn() {
    let arc: Vec<_> = (0..20_u32)
        .map(|step| {
            let angle = f64::from(step) / 10.0;
            (300.0 + 200.0 * angle.cos(), 300.0 + 200.0 * angle.sin())
        })
        .collect();
    assert!((track(&arc).median_turn() - 0.1).abs() < 1e-9);
}

#[test]
fn a_spot_lit_twice_in_place_does_not_count_as_a_turn() {
    let points = [
        (50.0, 100.0),
        (50.0, 90.0),
        (50.0, 90.0),
        (50.0, 80.0),
        (50.0, 70.0),
        (50.0, 60.0),
    ];
    assert_eq!(track(&points).median_turn(), 0.0);
}

#[test]
fn tracks_too_short_to_turn_have_no_turn_for_lack_of_evidence() {
    assert_eq!(track(&[(0.0, 0.0)]).median_turn(), 0.0);
    assert_eq!(track(&[(0.0, 0.0), (10.0, 0.0)]).median_turn(), 0.0);
}

#[test]
fn two_sharp_turns_out_of_three_give_a_right_angle() {
    let points = [(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (40.0, 20.0), (60.0, 20.0)];
    assert!((track(&points).median_turn() - FRAC_PI_2).abs() < 1e-12);
}

#[test]
fn one_sharp_turn_in_a_straight_track_leaves_the_median_at_zero() {
    let points = [
        (0.0, 0.0),
        (20.0, 0.0),
        (40.0, 0.0),
        (40.0, 20.0),
        (40.0, 40.0),
        (40.0, 60.0),
    ];
    assert_eq!(track(&points).median_turn(), 0.0);
}

#[test]
fn a_gentle_zigzag_flying_west_is_still_gentle_across_the_half_turn_boundary() {
    let east = zigzag(2.0, 1.0).median_turn();
    assert!((zigzag(2.0, -1.0).median_turn() - east).abs() < 1e-12);
    assert!(east < 0.8);
}

#[test]
fn a_wide_zigzag_turns_more_than_a_gentle_one() {
    assert!(zigzag(15.0, 1.0).median_turn() > 0.8);
}

#[test]
fn an_even_number_of_turns_takes_the_mean_of_the_middle_two() {
    assert_eq!(median(&[0.0, 0.5, 1.5, PI]), 1.0);
    assert_eq!(median(&[0.2]), 0.2);
    assert_eq!(median(&[]), 0.0);
}
