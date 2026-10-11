use std::num::NonZeroU32;

use super::{TRACK_LABEL, style_for};
use crate::text::{Typeface, Weight};

fn width(pixels: u32) -> NonZeroU32 {
    NonZeroU32::new(pixels).unwrap()
}

#[test]
fn the_summary_of_a_1440_wide_video_keeps_the_v011_proportions() {
    let typeface = Typeface::roboto().unwrap();
    let style = style_for(width(1440), "99", &typeface);
    assert_eq!(
        (
            style.line,
            style.border,
            style.outline,
            style.arrow_length,
            style.margin
        ),
        (3.0, 2.0, 7.0, 30.0, 16.0)
    );
    assert_eq!((style.radius, style.text.digit_px()), (23.0, 21.0));
    assert_eq!(
        (style.text.weight(), style.label.weight()),
        (Weight::Regular, Weight::SemiBold)
    );
    assert!((style.neighbor_distance - 240.0).abs() < 1e-4);
}

#[test]
fn a_narrow_video_keeps_the_minimum_sizes() {
    let typeface = Typeface::roboto().unwrap();
    let style = style_for(width(320), "9", &typeface);
    assert_eq!(
        (style.line, style.border, style.radius, style.arrow_length, style.margin),
        (1.0, 1.0, 8.0, 8.0, 4.0)
    );
    assert_eq!(style.text.digit_px(), 8.0);
}

#[test]
fn a_wide_label_enlarges_the_marker_so_it_fits() {
    let typeface = Typeface::roboto().unwrap();
    let style = style_for(width(320), "99999", &typeface);
    let label_width = typeface.measure("99999", style.label);
    assert!(style.radius > 8.0);
    assert!(2.0 * style.radius >= label_width + 4.0 * style.border);
}

#[test]
fn track_numbers_stay_semibold_at_17_pixels_whatever_the_video_size() {
    assert_eq!((TRACK_LABEL.digit_px(), TRACK_LABEL.weight()), (17.0, Weight::SemiBold));
}
