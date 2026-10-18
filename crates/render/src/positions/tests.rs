use types::detection::{Detection, Strength};
use types::frame::FrameIndex;

use super::{bounded, filled};

fn point(frame: usize, width: f64, height: f64) -> Detection {
    Detection {
        frame: FrameIndex::new(frame),
        x: 100.0,
        y: 50.0,
        left: 100.0 - width / 2.0,
        top: 50.0 - height / 2.0,
        width,
        height,
        area: width * height,
        amplitude: 50.0,
        strength: Strength::Strong,
    }
}

#[test]
fn a_side_spread_beyond_four_medians_is_brought_back_around_the_center() {
    let points = [point(0, 10.0, 6.0), point(1, 10.0, 6.0), point(2, 80.0, 6.0)];
    let kept = bounded(&points);
    assert_eq!((kept[2].left, kept[2].width), (80.0, 40.0));
    assert_eq!((kept[2].top, kept[2].height), (47.0, 6.0));
    assert_eq!(kept[..2], points[..2]);
}

#[test]
fn the_frames_between_two_points_are_filled_and_marked_interpolated() {
    let filled = filled(&[point(0, 10.0, 6.0), point(3, 16.0, 6.0)]);
    let frames: Vec<(usize, bool)> = filled
        .iter()
        .map(|(point, interpolated)| (point.frame.get(), *interpolated))
        .collect();
    assert_eq!(frames, [(0, false), (1, true), (2, true), (3, false)]);
    assert_eq!(filled[1].0.width, 12.0);
}
