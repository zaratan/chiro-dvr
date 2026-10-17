use types::detection::{FrameDetections, Measures, Strength};

use super::{BESIDE_STRONG, BRACKET_TOLERANCE, SUPPORT_FRAMES, SUPPORT_MOVE, SUPPORT_STEP, bracketed};

fn measures(x: f64, y: f64, strength: Strength) -> Measures {
    Measures {
        x,
        y,
        left: x - 2.0,
        top: y - 2.0,
        width: 4.5,
        height: 4.5,
        area: 9.0,
        amplitude: 11.0,
        strength,
    }
}

fn strong(x: f64) -> Measures {
    measures(x, 100.0, Strength::Strong)
}

fn weak(x: f64) -> Measures {
    measures(x, 100.0, Strength::Weak)
}

fn frames(found: Vec<Vec<Measures>>) -> FrameDetections {
    let mut frames = FrameDetections::default();
    for measures in found {
        frames.push(measures);
    }
    frames
}

fn weak_kept(found: Vec<Vec<Measures>>) -> usize {
    bracketed(&frames(found))
        .iter()
        .flat_map(|(_, found)| found)
        .filter(|detection| detection.strength == Strength::Weak)
        .count()
}

#[test]
fn a_weak_detection_on_the_way_between_a_strong_one_before_and_after_is_kept() {
    assert_eq!(
        weak_kept(vec![vec![strong(0.0)], vec![weak(30.0)], vec![strong(60.0)]]),
        1
    );
}

#[test]
fn a_weak_detection_seen_from_one_side_only_is_dropped() {
    assert_eq!(
        weak_kept(vec![vec![strong(0.0)], vec![strong(30.0)], vec![weak(60.0)]]),
        0
    );
    assert_eq!(
        weak_kept(vec![vec![weak(0.0)], vec![strong(30.0)], vec![strong(60.0)]]),
        0
    );
}

#[test]
fn a_support_more_than_three_frames_away_does_not_count() {
    let mut found = vec![vec![strong(0.0)], vec![weak(10.0)]];
    found.extend((0..SUPPORT_FRAMES).map(|_| Vec::new()));
    found.push(vec![strong(10.0 * (SUPPORT_FRAMES + 2) as f64)]);
    assert_eq!(weak_kept(found.clone()), 0);
    found.remove(2);
    assert_eq!(weak_kept(found), 1);
}

#[test]
fn a_support_farther_than_sixty_pixels_per_frame_does_not_count() {
    let far = SUPPORT_STEP + 1.0;
    assert_eq!(
        weak_kept(vec![vec![strong(0.0)], vec![weak(far)], vec![strong(2.0 * far)]]),
        0
    );
    let near = SUPPORT_STEP - 1.0;
    assert_eq!(
        weak_kept(vec![vec![strong(0.0)], vec![weak(near)], vec![strong(2.0 * near)]]),
        1
    );
}

#[test]
fn supports_that_move_less_than_twenty_pixels_keep_nothing_since_a_still_source_is_no_flight() {
    let short = SUPPORT_MOVE - 1.0;
    assert_eq!(
        weak_kept(vec![vec![strong(0.0)], vec![weak(short / 2.0)], vec![strong(short)]]),
        0
    );
}

#[test]
fn a_weak_detection_more_than_ten_pixels_off_the_interpolated_place_is_dropped() {
    let off = |shift: f64| {
        vec![
            vec![strong(0.0)],
            vec![measures(30.0, 100.0 + shift, Strength::Weak)],
            vec![strong(60.0)],
        ]
    };
    assert_eq!(weak_kept(off(BRACKET_TOLERANCE + 0.5)), 0);
    assert_eq!(weak_kept(off(BRACKET_TOLERANCE - 0.5)), 1);
}

#[test]
fn the_interpolated_place_weighs_the_frames_on_each_side() {
    assert_eq!(
        weak_kept(vec![vec![strong(0.0)], vec![weak(20.0)], vec![], vec![strong(60.0)]]),
        1
    );
    assert_eq!(
        weak_kept(vec![vec![strong(0.0)], vec![weak(40.0)], vec![], vec![strong(60.0)]]),
        0
    );
}

#[test]
fn a_weak_detection_beside_a_strong_one_of_its_frame_is_dropped_as_a_piece_of_it() {
    let beside = vec![
        weak(30.0),
        measures(30.0, 100.0 + BESIDE_STRONG - 1.0, Strength::Strong),
    ];
    assert_eq!(weak_kept(vec![vec![strong(0.0)], beside, vec![strong(60.0)]]), 0);
}

#[test]
fn weak_detections_do_not_support_each_other() {
    assert_eq!(weak_kept(vec![vec![weak(0.0)], vec![weak(30.0)], vec![weak(60.0)]]), 0);
}

#[test]
fn strong_detections_are_all_kept_unchanged_in_their_frames() {
    let found = vec![vec![strong(0.0), strong(500.0)], vec![], vec![strong(900.0)]];
    let kept = bracketed(&frames(found.clone()));
    let back: Vec<Vec<Measures>> = kept
        .iter()
        .map(|(_, found)| found.iter().map(Measures::of).collect())
        .collect();
    assert_eq!(back, found);
}

#[test]
fn two_weak_detections_in_a_row_between_the_same_supports_are_both_kept() {
    assert_eq!(
        weak_kept(vec![
            vec![strong(0.0)],
            vec![weak(20.0)],
            vec![weak(40.0)],
            vec![strong(60.0)]
        ]),
        2
    );
}

#[test]
fn a_weak_detection_just_beyond_twelve_pixels_of_a_strong_one_of_its_frame_is_kept() {
    let apart = vec![
        weak(30.0),
        measures(30.0, 100.0 + BESIDE_STRONG + 1.0, Strength::Strong),
    ];
    assert_eq!(weak_kept(vec![vec![strong(0.0)], apart, vec![strong(60.0)]]), 1);
}
