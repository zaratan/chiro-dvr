use reference::frames::{FrameEntry, Frames};
use types::detection::{FrameDetections, Measures};
use types::frame::{FrameIndex, Pts};
use types::span::Span;

use super::{LISTED_FRAMES, Sameness, Verdict};
use crate::inputs::{Field, InputGap};

const VIDEO: &str = "video_101.mp4";

fn entry(frame: usize) -> FrameEntry {
    FrameEntry {
        pts: Some(Pts::new(100 * frame as i64)),
        key_frame: frame == 0,
        decode_error: false,
    }
}

fn measures(x: f64) -> Measures {
    Measures {
        x,
        y: 10.0,
        left: x - 1.0,
        top: 9.0,
        width: 3.0,
        height: 3.0,
        area: 4.5,
        amplitude: 13.0,
    }
}

fn run(frame_count: usize, moved: &[usize]) -> Frames {
    let mut detections = FrameDetections::default();
    for frame in 0..frame_count {
        let shift = if moved.contains(&frame) { 0.5 } else { 0.0 };
        detections.push([measures(100.0 + shift)]);
    }
    Frames::new((0..frame_count).map(entry).collect(), detections).unwrap()
}

fn with_entry(frames: &Frames, frame: usize, changed: FrameEntry) -> Frames {
    let mut entries = frames.entries().to_vec();
    entries[frame] = changed;
    Frames::new(entries, frames.detections().clone()).unwrap()
}

fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

fn line(sameness: &Sameness) -> String {
    let mut out = Vec::new();
    sameness.write_line(&mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn two_runs_alike_on_every_frame_are_identical() {
    let sameness = Sameness::of(VIDEO, &run(20, &[]), &run(20, &[]), &[span(4, 6)]);

    assert!(sameness.holds());
    assert_eq!(
        line(&sameness),
        "video_101.mp4\tidentical\tspans 1 (3 frames)\tdiffering inside 0\n"
    );
}

#[test]
fn detections_that_differ_only_inside_the_damaged_spans_are_counted_but_leave_the_runs_identical() {
    let damaged = [span(4, 6), span(15, 15)];

    let sameness = Sameness::of(VIDEO, &run(20, &[]), &run(20, &[4, 6, 15]), &damaged);

    assert!(sameness.holds());
    assert_eq!(
        line(&sameness),
        "video_101.mp4\tidentical\tspans 2 (4 frames)\tdiffering inside 3\n"
    );
}

#[test]
fn a_detection_changed_outside_the_damaged_spans_is_listed_with_its_video_and_frame() {
    let sameness = Sameness::of(VIDEO, &run(20, &[]), &run(20, &[3, 5, 7]), &[span(4, 6)]);

    assert!(!sameness.holds());
    assert_eq!(
        line(&sameness),
        "video_101.mp4\tdiffers\toutside 2: 3, 7\tspans 1 (3 frames)\tdiffering inside 1\n"
    );
}

#[test]
fn a_detection_lost_on_one_side_differs_as_much_as_a_moved_one() {
    let previous = run(3, &[]);
    let mut fewer = FrameDetections::default();
    fewer.push([measures(100.0)]);
    fewer.push([]);
    fewer.push([measures(100.0)]);
    let current = Frames::new(previous.entries().to_vec(), fewer).unwrap();

    let sameness = Sameness::of(VIDEO, &previous, &current, &[]);

    assert_eq!(
        sameness.verdict(),
        &Verdict::Compared {
            outside: vec![FrameIndex::new(1)],
            differing_inside: 0,
            span_count: 0,
            damaged_frames: 0
        }
    );
}

#[test]
fn past_ten_frames_the_list_stops_and_tells_how_many_more_differ() {
    let moved: Vec<usize> = (0..LISTED_FRAMES + 3).collect();

    let sameness = Sameness::of(VIDEO, &run(20, &[]), &run(20, &moved), &[]);

    assert_eq!(
        line(&sameness),
        "video_101.mp4\tdiffers\toutside 13: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 … and 3 more\tspans 0 (0 frames)\tdiffering inside 0\n"
    );
}

#[test]
fn inputs_that_differ_are_told_at_their_first_frame_with_every_field_and_stop_the_comparison() {
    let previous = run(10, &[]);
    let current = with_entry(
        &with_entry(
            &run(10, &[1]),
            5,
            FrameEntry {
                pts: None,
                key_frame: false,
                decode_error: true,
            },
        ),
        8,
        FrameEntry {
            key_frame: true,
            ..entry(8)
        },
    );

    let sameness = Sameness::of(VIDEO, &previous, &current, &[]);

    assert!(!sameness.holds());
    assert_eq!(
        sameness.verdict(),
        &Verdict::InputsDiffer(InputGap::Entry {
            frame: FrameIndex::new(5),
            fields: vec![Field::Pts, Field::DecodeError]
        })
    );
    assert_eq!(
        line(&sameness),
        "video_101.mp4\tinputs differ\tframe 5: pts, decode error\n"
    );
}

#[test]
fn a_key_frame_seen_on_one_side_only_is_an_input_gap_even_inside_a_damaged_span() {
    let current = with_entry(
        &run(10, &[]),
        4,
        FrameEntry {
            key_frame: true,
            ..entry(4)
        },
    );

    let sameness = Sameness::of(VIDEO, &run(10, &[]), &current, &[span(2, 6)]);

    assert_eq!(line(&sameness), "video_101.mp4\tinputs differ\tframe 4: key frame\n");
}

#[test]
fn runs_of_different_lengths_differ_by_their_inputs() {
    let sameness = Sameness::of(VIDEO, &run(10, &[]), &run(9, &[]), &[]);

    assert_eq!(
        sameness.verdict(),
        &Verdict::InputsDiffer(InputGap::FrameCount {
            previous: 10,
            current: 9
        })
    );
    assert_eq!(
        line(&sameness),
        "video_101.mp4\tinputs differ\t10 frames before, 9 now\n"
    );
}
