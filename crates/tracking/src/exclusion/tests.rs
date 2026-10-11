use std::num::NonZeroU32;

use types::detection::FrameDetections;
use types::rate::FrameRate;
use types::rational::Rational;
use types::span::Span;

use super::{Exclusion, exclude};
use crate::fake::{crowd, span};
use crate::stability::StabilityConfig;

const FRAMES: usize = 60;

fn counts(crowded: &[(usize, usize)], usual: usize) -> FrameDetections {
    let counts: Vec<usize> = (0..FRAMES)
        .map(|frame| {
            crowded
                .iter()
                .find(|&&(at, _)| at == frame)
                .map_or(usual, |&(_, count)| count)
        })
        .collect();
    crowd(&counts)
}

fn run(detections: &FrameDetections, damaged: Vec<Span>) -> Exclusion {
    let rate = FrameRate::new(Rational::new(10, 1).unwrap()).unwrap();
    let stability = StabilityConfig::new(NonZeroU32::new(20), 0.2).unwrap();
    exclude(detections, damaged, 1, rate, stability).unwrap()
}

fn emptied(exclusion: &Exclusion) -> Vec<usize> {
    exclusion
        .detections
        .iter()
        .filter(|(_, found)| found.is_empty())
        .map(|(frame, _)| frame.get())
        .collect()
}

#[test]
fn damaged_frames_cannot_make_a_span_unstable() {
    let exclusion = run(&counts(&[(17, 200), (18, 200)], 1), vec![span(17, 29)]);
    assert_eq!(exclusion.unstable, []);
}

#[test]
fn damaged_frames_do_not_lower_the_usual_blob_count() {
    let mut crowded: Vec<(usize, usize)> = (0..45).map(|frame| (frame, 0)).collect();
    crowded.push((50, 79));
    let damaged = vec![span(0, 14), span(15, 29), span(30, 44)];
    assert_eq!(run(&counts(&crowded, 10), damaged).unstable, []);
}

#[test]
fn detections_inside_damaged_spans_are_dropped_but_frames_remain() {
    let exclusion = run(&counts(&[], 1), vec![span(33, 44)]);
    assert_eq!(exclusion.detections.frame_count(), FRAMES);
    assert_eq!(emptied(&exclusion), (33..45).collect::<Vec<_>>());
}

#[test]
fn real_camera_movement_outside_damage_is_still_unstable_and_emptied() {
    let exclusion = run(&counts(&[(17, 200), (50, 200)], 1), vec![span(17, 29)]);
    assert_eq!(exclusion.unstable, [span(48, 52)]);
    assert_eq!(
        exclusion.ignored_spans().collect::<Vec<_>>(),
        [span(17, 29), span(48, 52)]
    );
    assert_eq!(emptied(&exclusion), (17..30).chain(48..53).collect::<Vec<_>>());
}

#[test]
fn time_both_damaged_and_unstable_is_not_counted_twice() {
    let apart = run(&counts(&[(33, 200)], 1), vec![span(17, 29)]);
    assert_eq!(apart.ignored_spans().collect::<Vec<_>>(), [span(17, 29), span(31, 35)]);
    assert_eq!(apart.ignored_frame_count, 13 + 5);
    let overlapping = run(&counts(&[(31, 200)], 1), vec![span(17, 29)]);
    assert_eq!(
        overlapping.ignored_spans().collect::<Vec<_>>(),
        [span(17, 29), span(29, 33)]
    );
    assert_eq!(overlapping.ignored_frame_count, 17);
    assert_eq!(overlapping.damaged_frame_count, 13);
}

#[test]
fn the_reported_damage_and_the_damaged_spans_are_handed_back() {
    let exclusion = run(&counts(&[], 1), vec![span(17, 29), span(40, 44)]);
    assert_eq!(exclusion.reported_frames, 1);
    assert_eq!(exclusion.damaged, [span(17, 29), span(40, 44)]);
}

#[test]
fn crowded_frames_on_both_ends_of_a_damaged_span_are_hidden() {
    let exclusion = run(&counts(&[(14, 200), (32, 200)], 1), vec![span(14, 32)]);
    assert_eq!(exclusion.unstable, []);
}
