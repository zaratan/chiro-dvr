use super::{Detection, FrameDetections, Measures, Strength};
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
        strength: Strength::Strong,
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
        strength: Strength::Strong,
    }
}

#[test]
fn the_distance_between_detections_is_euclidean() {
    assert!((detection(0, 0.0, 0.0).distance(&detection(0, 3.0, 4.0)) - 5.0).abs() < 1e-12);
}

#[test]
fn empty_frames_keep_their_place() {
    let mut frames = FrameDetections::default();
    frames.push([]);
    frames.push([measures(5.0)]);
    frames.push([]);
    assert_eq!((frames.frame_count(), frames.detection_count()), (3, 1));
    assert_eq!(frames.get(FrameIndex::new(2)), Some([].as_slice()));
    assert_eq!(frames.get(FrameIndex::new(3)), None);
}

#[test]
fn detections_of_a_frame_keep_their_order() {
    let mut frames = FrameDetections::default();
    frames.push([measures(900.0), measures(10.0)]);
    assert_eq!(
        frames.get(FrameIndex::new(0)),
        Some([detection(0, 900.0, 10.0), detection(0, 10.0, 10.0)].as_slice())
    );
}

#[test]
fn iteration_walks_every_frame_in_order() {
    let mut frames = FrameDetections::default();
    frames.push([measures(1.0)]);
    frames.push([]);
    let counts: Vec<_> = frames.iter().map(|(frame, found)| (frame.get(), found.len())).collect();
    assert_eq!(counts, [(0, 1), (1, 0)]);
}

fn one_per_frame(count: usize) -> FrameDetections {
    let mut frames = FrameDetections::default();
    for _ in 0..count {
        frames.push([measures(1.0)]);
    }
    frames
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

#[test]
fn detections_pushed_frame_after_frame_take_the_frame_of_the_place_they_fill() {
    let mut frames = FrameDetections::default();

    frames.push([measures(5.0), measures(7.0)]);
    frames.push([]);
    frames.push([measures(9.0)]);

    assert_eq!(counts(&frames), [2, 0, 1]);
    assert_eq!(
        frames.get(FrameIndex::new(2)),
        Some([measures(9.0).at(FrameIndex::new(2))].as_slice())
    );
    assert!(
        frames
            .iter()
            .all(|(frame, found)| found.iter().all(|detection| detection.frame == frame))
    );
}

#[test]
fn measures_placed_on_a_frame_keep_every_measure() {
    assert_eq!(measures(5.0).at(FrameIndex::new(3)), detection(3, 5.0, 10.0));
}

#[test]
fn the_measures_of_a_detection_placed_back_on_its_frame_give_the_detection_again() {
    let found = Detection {
        height: 1.5,
        ..detection(7, 1400.5, 246.25)
    };

    assert_eq!(Measures::of(&found).at(found.frame), found);
}

#[test]
fn measures_read_from_a_line_of_the_v011_are_written_back_byte_for_byte() {
    let line = r#"{"x":338.8792134831461,"y":39.21629213483146,"left":333.0,"top":30.0,"width":13.5,"height":21.0,"area":155.25,"amplitude":96.77308824327257}"#;

    let read: Measures = serde_json::from_str(line).unwrap();

    assert_eq!(serde_json::to_string(&read).unwrap(), line);
}

#[test]
fn measures_written_before_weak_detections_existed_are_read_back_strong() {
    let line = r#"{"x":1.5,"y":2.0,"left":1.0,"top":1.5,"width":1.5,"height":1.5,"area":2.25,"amplitude":13.5}"#;
    let read: Measures = serde_json::from_str(line).unwrap();
    assert_eq!(read.strength, Strength::Strong);
}

#[test]
fn a_weak_detection_writes_its_strength_and_reads_it_back() {
    let weak = Measures {
        strength: Strength::Weak,
        ..measures(5.0)
    };
    let line = serde_json::to_string(&weak).unwrap();
    assert!(line.ends_with(r#","strength":"weak"}"#), "{line}");
    assert_eq!(serde_json::from_str::<Measures>(&line).unwrap(), weak);
}

#[test]
fn two_detections_joined_take_the_stronger_strength() {
    assert_eq!(Strength::Weak.stronger(Strength::Strong), Strength::Strong);
    assert_eq!(Strength::Strong.stronger(Strength::Weak), Strength::Strong);
    assert_eq!(Strength::Weak.stronger(Strength::Weak), Strength::Weak);
}
