use proptest::prelude::*;
use types::detection::{Detection, FrameDetections, Measures, Strength};
use types::frame::FrameIndex;

use super::{DETECTION_DISTANCE_PX, match_detections};
use crate::location::{Reference, committed};
use crate::recording::Recording;

fn detections_092() -> FrameDetections {
    Recording::read(&committed(Reference::V011, "video_092_original"))
        .unwrap()
        .frames
        .detections()
        .clone()
}

fn detection(frame: usize, x: f64, y: f64) -> Detection {
    Detection {
        frame: FrameIndex::new(frame),
        x,
        y,
        left: x - 1.0,
        top: y - 1.0,
        width: 3.0,
        height: 3.0,
        area: 4.5,
        amplitude: 13.0,
        strength: Strength::Strong,
    }
}

fn frames(frames: Vec<Vec<Detection>>) -> FrameDetections {
    let mut placed = FrameDetections::default();
    for found in frames {
        placed.push(found.iter().map(Measures::of));
    }
    placed
}

fn shifted_first_detection(detections: &FrameDetections, dx: f64) -> (FrameDetections, Detection, Detection) {
    let mut moved = None;
    let rows: Vec<Vec<Detection>> = detections
        .iter()
        .map(|(_, found)| {
            found
                .iter()
                .map(|&detection| {
                    if moved.is_some() {
                        return detection;
                    }
                    let shifted = Detection {
                        x: detection.x + dx,
                        ..detection
                    };
                    moved = Some((detection, shifted));
                    shifted
                })
                .collect()
        })
        .collect();
    let (original, shifted) = moved.unwrap();
    (frames(rows), original, shifted)
}

#[test]
fn the_reference_matches_itself_completely() {
    let reference = detections_092();
    let matching = match_detections(&reference, &reference);
    assert_eq!(matching.pairs.len(), 780);
    assert!(matching.is_complete());
}

#[test]
fn a_detection_moved_by_2_9_px_still_matches() {
    let reference = detections_092();
    let (candidate, _, _) = shifted_first_detection(&reference, 2.9);
    let matching = match_detections(&reference, &candidate);
    assert_eq!(matching.pairs.len(), 780);
    assert!(matching.is_complete());
}

#[test]
fn a_detection_moved_by_3_px_no_longer_matches() {
    let reference = detections_092();
    let (candidate, original, shifted) = shifted_first_detection(&reference, 3.0);
    let matching = match_detections(&reference, &candidate);
    assert_eq!(matching.pairs.len(), 779);
    assert_eq!(
        (matching.reference_only, matching.candidate_only),
        (vec![original], vec![shifted])
    );
}

#[test]
fn a_detection_matches_at_most_once_and_the_closest_wins() {
    let reference = frames(vec![vec![detection(0, 100.0, 100.0)]]);
    let far = detection(0, 102.0, 100.0);
    let near = detection(0, 101.0, 100.0);
    let matching = match_detections(&reference, &frames(vec![vec![far, near]]));
    assert_eq!(matching.pairs, [(reference.get(FrameIndex::new(0)).unwrap()[0], near)]);
    assert_eq!(matching.candidate_only, [far]);
}

#[test]
fn the_same_place_on_another_frame_does_not_match() {
    let reference = frames(vec![vec![detection(0, 100.0, 100.0)], vec![]]);
    let candidate = frames(vec![vec![], vec![detection(1, 100.0, 100.0)]]);
    let matching = match_detections(&reference, &candidate);
    assert_eq!(matching.pairs.len(), 0);
    assert_eq!((matching.reference_only.len(), matching.candidate_only.len()), (1, 1));
}

#[test]
fn detections_past_the_end_of_the_other_side_are_unmatched() {
    let reference = frames(vec![vec![]]);
    let candidate = frames(vec![vec![], vec![detection(1, 5.0, 5.0)]]);
    assert_eq!(match_detections(&reference, &candidate).candidate_only.len(), 1);
    assert_eq!(match_detections(&candidate, &reference).reference_only.len(), 1);
}

fn oracle_over_every_pair(reference: &[Detection], candidate: &[Detection]) -> Vec<(usize, usize)> {
    let mut close: Vec<(f64, usize, usize)> = reference
        .iter()
        .enumerate()
        .flat_map(|(r, kept)| {
            candidate
                .iter()
                .enumerate()
                .map(move |(c, found)| (kept.distance(found), r, c))
        })
        .filter(|&(distance, _, _)| distance < DETECTION_DISTANCE_PX)
        .collect();
    close.sort_by(|a, b| a.0.total_cmp(&b.0).then((a.1, a.2).cmp(&(b.1, b.2))));
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for (_, r, c) in close {
        if pairs.iter().all(|&(taken_r, taken_c)| taken_r != r && taken_c != c) {
            pairs.push((r, c));
        }
    }
    pairs
}

fn assert_same_as_every_pair(reference: &[Detection], candidate: &[Detection]) -> Result<(), TestCaseError> {
    let expected = oracle_over_every_pair(reference, candidate);
    let matching = match_detections(&frames(vec![reference.to_vec()]), &frames(vec![candidate.to_vec()]));
    let expected_pairs: Vec<_> = expected.iter().map(|&(r, c)| (reference[r], candidate[c])).collect();
    let alone = |side: &[Detection], taken: Vec<usize>| -> Vec<Detection> {
        side.iter()
            .enumerate()
            .filter(|(index, _)| !taken.contains(index))
            .map(|(_, detection)| *detection)
            .collect()
    };
    prop_assert_eq!(matching.pairs, expected_pairs);
    prop_assert_eq!(
        matching.reference_only,
        alone(reference, expected.iter().map(|pair| pair.0).collect())
    );
    prop_assert_eq!(
        matching.candidate_only,
        alone(candidate, expected.iter().map(|pair| pair.1).collect())
    );
    Ok(())
}

fn crowd() -> impl Strategy<Value = Vec<Detection>> {
    prop::collection::vec((0.0..20.0_f64, 0.0..20.0_f64), 0..40)
        .prop_map(|positions| positions.into_iter().map(|(x, y)| detection(0, x, y)).collect())
}

fn grid() -> impl Strategy<Value = Vec<Detection>> {
    prop::collection::vec((0..12_u8, 0..12_u8), 0..40).prop_map(|positions| {
        positions
            .into_iter()
            .map(|(x, y)| detection(0, f64::from(x), f64::from(y)))
            .collect()
    })
}

#[test]
fn exactly_3_px_apart_does_not_match() {
    let reference = frames(vec![vec![detection(0, 100.0, 100.0)]]);
    let candidate = frames(vec![vec![detection(0, 103.0, 100.0)]]);
    assert_eq!(match_detections(&reference, &candidate).pairs, []);
}

#[test]
fn a_detection_without_a_finite_position_stays_alone_without_blocking_the_others() {
    let reference = frames(vec![vec![detection(0, 10.0, 10.0)]]);
    let lost = detection(0, -f64::NAN, 10.0);
    let near = detection(0, 11.0, 10.0);
    let matching = match_detections(&reference, &frames(vec![vec![lost, near]]));
    assert_eq!(matching.pairs.len(), 1);
    assert!(matching.candidate_only[0].x.is_nan());
}

proptest! {
    #[test]
    fn looking_only_at_close_columns_matches_like_trying_every_pair(reference in crowd(), candidate in crowd()) {
        assert_same_as_every_pair(&reference, &candidate)?;
    }

    #[test]
    fn ties_and_the_3_px_bound_on_whole_pixels_match_like_trying_every_pair(reference in grid(), candidate in grid()) {
        assert_same_as_every_pair(&reference, &candidate)?;
    }
}
