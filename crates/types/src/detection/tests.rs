use super::{Detection, Error, FrameDetections};
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
