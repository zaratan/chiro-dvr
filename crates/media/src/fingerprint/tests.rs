use super::{of, of_pixels, of_sequence};
use crate::frame::{ColorRange, YuvFrame};

const STORED_GRAY: u64 = 5_997_605_158_854_829_352;

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

#[test]
fn a_stored_fingerprint_stays_valid_across_builds_and_systems() {
    assert_eq!(of(&gray()), STORED_GRAY);
}

#[test]
fn pixels_of_another_size_or_one_changed_byte_are_another_image() {
    let pixels = [9; 4 * 2 * 4];
    let mut changed = pixels;
    changed[31] = 10;

    let reference = of_pixels(4, 2, &pixels);

    assert_eq!(of_pixels(4, 2, &pixels), reference);
    for other in [
        of_pixels(2, 4, &pixels),
        of_pixels(4, 2, &changed),
        of_pixels(4, 2, &pixels[..28]),
    ] {
        assert_ne!(other, reference);
    }
}

#[test]
fn a_sequence_of_fingerprints_depends_on_their_order_and_number() {
    let reference = of_sequence([1, 2, 3]);

    assert_eq!(of_sequence([1, 2, 3]), reference);
    for other in [of_sequence([2, 1, 3]), of_sequence([1, 2]), of_sequence([1, 2, 3, 0])] {
        assert_ne!(other, reference);
    }
}

#[test]
fn two_words_changed_only_in_their_high_byte_do_not_cancel() {
    let pixels = [0_u8; 16];
    let mut changed = pixels;
    changed[7] ^= 0x80;
    changed[15] ^= 0x80;

    assert_ne!(of_pixels(2, 2, &changed), of_pixels(2, 2, &pixels));
}
