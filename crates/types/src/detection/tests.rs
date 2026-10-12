use super::{Detection, Error, FrameDetections};
use crate::frame::FrameIndex;
use crate::span::Span;

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
    }
}

#[test]
fn the_distance_between_detections_is_euclidean() {
    assert!((detection(0, 0.0, 0.0).distance(&detection(0, 3.0, 4.0)) - 5.0).abs() < 1e-12);
}

#[test]
fn empty_frames_keep_their_place() {
    let frames = FrameDetections::new(vec![vec![], vec![detection(1, 5.0, 5.0)], vec![]]).unwrap();
    assert_eq!((frames.frame_count(), frames.detection_count()), (3, 1));
    assert_eq!(frames.get(FrameIndex::new(2)), Some([].as_slice()));
    assert_eq!(frames.get(FrameIndex::new(3)), None);
}

#[test]
fn detections_of_a_frame_keep_their_order() {
    let first = detection(0, 900.0, 10.0);
    let second = detection(0, 10.0, 10.0);
    let frames = FrameDetections::new(vec![vec![first, second]]).unwrap();
    assert_eq!(frames.get(FrameIndex::new(0)), Some([first, second].as_slice()));
}

#[test]
fn a_detection_listed_under_another_frame_is_refused() {
    let error = FrameDetections::new(vec![vec![], vec![detection(0, 1.0, 1.0)]]).unwrap_err();
    assert_eq!(
        error,
        Error::Misplaced {
            slot: FrameIndex::new(1),
            found: FrameIndex::new(0)
        }
    );
    assert_eq!(error.to_string(), "a detection of frame 0 is listed under frame 1");
}

#[test]
fn iteration_walks_every_frame_in_order() {
    let frames = FrameDetections::new(vec![vec![detection(0, 1.0, 1.0)], vec![]]).unwrap();
    let counts: Vec<_> = frames.iter().map(|(frame, found)| (frame.get(), found.len())).collect();
    assert_eq!(counts, [(0, 1), (1, 0)]);
}

fn one_per_frame(count: usize) -> FrameDetections {
    FrameDetections::new((0..count).map(|frame| vec![detection(frame, 1.0, 1.0)]).collect()).unwrap()
}

fn counts(frames: &FrameDetections) -> Vec<usize> {
    frames.iter().map(|(_, found)| found.len()).collect()
}

#[test]
fn removing_spans_empties_their_frames_but_keeps_them() {
    let spans = [
        Span::single(FrameIndex::new(1)),
        Span::new(FrameIndex::new(3), FrameIndex::new(4)).unwrap(),
    ];
    let kept = one_per_frame(6).without(&spans);
    assert_eq!(counts(&kept), [1, 0, 1, 0, 0, 1]);
    assert_eq!(kept.get(FrameIndex::new(5)), one_per_frame(6).get(FrameIndex::new(5)));
}

#[test]
fn a_span_running_past_the_last_frame_empties_up_to_it() {
    let spans = [
        Span::new(FrameIndex::new(2), FrameIndex::new(40)).unwrap(),
        Span::single(FrameIndex::new(50)),
    ];
    assert_eq!(counts(&one_per_frame(4).without(&spans)), [1, 1, 0, 0]);
    let endless = [Span::single(FrameIndex::new(2)).widened(0, usize::MAX)];
    assert_eq!(counts(&one_per_frame(4).without(&endless)), [1, 1, 0, 0]);
}

#[test]
fn removing_no_span_keeps_every_detection() {
    assert_eq!(one_per_frame(3).without(&[]), one_per_frame(3));
}

#[test]
fn many_short_spans_empty_exactly_their_frames() {
    let spans: Vec<Span> = (0..10_000)
        .step_by(4)
        .map(|first| Span::new(FrameIndex::new(first), FrameIndex::new(first + 1)).unwrap())
        .collect();
    let left = one_per_frame(10_000).without(&spans);
    let first_left: Vec<usize> = left
        .iter()
        .filter(|(_, found)| !found.is_empty())
        .map(|(frame, _)| frame.get())
        .take(4)
        .collect();
    assert_eq!(first_left, [2, 3, 6, 7]);
    assert_eq!(counts(&left).iter().filter(|&&count| count == 0).count(), 5_000);
}
