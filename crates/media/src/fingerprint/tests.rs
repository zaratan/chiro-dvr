use super::of;
use crate::frame::{ColorRange, YuvFrame};

fn gray() -> YuvFrame {
    uniform(4, 2, ColorRange::Limited)
}

fn uniform(width: u32, height: u32, range: ColorRange) -> YuvFrame {
    let mut frame = YuvFrame::new(width, height, range).unwrap();
    frame.y_mut().fill(100);
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    frame
}

#[test]
fn two_frames_with_the_same_pixels_have_the_same_fingerprint() {
    assert_eq!(of(&gray()), of(&gray()));
}

#[test]
fn one_sample_changed_in_any_plane_changes_the_fingerprint() {
    let reference = of(&gray());
    let mut luma = gray();
    luma.y_mut()[7] = 101;
    let mut blue = gray();
    blue.u_mut()[0] = 127;
    let mut red = gray();
    red.v_mut()[1] = 129;

    for changed in [luma, blue, red] {
        assert_ne!(of(&changed), reference);
    }
}

#[test]
fn the_blue_and_red_planes_are_not_interchangeable() {
    let mut first = gray();
    first.u_mut()[1] = 7;
    let mut second = gray();
    second.v_mut()[1] = 7;

    assert_ne!(of(&first), of(&second));
}

#[test]
fn the_same_samples_in_another_size_or_range_are_another_frame() {
    let reference = of(&gray());

    for other in [
        uniform(2, 4, ColorRange::Limited),
        uniform(4, 2, ColorRange::Full),
        uniform(4, 2, ColorRange::Unspecified),
    ] {
        assert_eq!((other.y(), other.u(), other.v()), (gray().y(), gray().u(), gray().v()));
        assert_ne!(of(&other), reference, "{other:?}");
    }
}
