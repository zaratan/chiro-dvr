use skrifa::GlyphId;

use super::{Error, REPLACEMENT, TextStyle, Typeface, Weight};

const TWENTY: TextStyle = TextStyle::trusted(20.0, Weight::Regular);

fn bounds(typeface: &Typeface, text: &str) -> Vec<(f32, f32, f32, f32)> {
    typeface
        .shape(text, TWENTY)
        .glyphs
        .iter()
        .map(|(path, left)| {
            let b = path.bounds();
            (left + b.left(), b.top(), b.width(), b.height())
        })
        .collect()
}

#[test]
fn every_digit_has_the_same_width_so_times_do_not_shift() {
    let typeface = Typeface::roboto().unwrap();
    let widths: Vec<f32> = ('0'..='9')
        .map(|digit| typeface.measure(&digit.to_string(), TWENTY))
        .collect();
    assert!(
        widths.iter().all(|width| (width - widths[0]).abs() < 1e-4),
        "{widths:?}"
    );
}

#[test]
fn a_decomposed_accent_is_laid_out_as_the_precomposed_letter() {
    let typeface = Typeface::roboto().unwrap();
    assert_eq!(bounds(&typeface, "e\u{301}"), bounds(&typeface, "é"));
}

#[test]
fn a_character_the_font_lacks_takes_the_replacement_glyph_not_the_empty_one() {
    let typeface = Typeface::roboto().unwrap();
    let replacement = typeface.glyph_id(REPLACEMENT);
    assert_ne!(replacement, GlyphId::NOTDEF);
    assert_eq!(typeface.glyph_id('\u{1F987}'), replacement);
    assert_eq!(bounds(&typeface, "\u{1F987}"), bounds(&typeface, "\u{FFFD}"));
}

#[test]
fn the_size_is_the_height_of_the_digits() {
    let typeface = Typeface::roboto().unwrap();
    let heights: Vec<f32> = bounds(&typeface, "0123456789")
        .into_iter()
        .map(|glyph| glyph.3)
        .collect();
    assert!((heights[0] - 20.0).abs() < 0.05, "{heights:?}");
    assert!(heights.iter().all(|height| (height - 20.0).abs() < 0.6), "{heights:?}");
}

#[test]
fn shaped_text_is_as_wide_as_measured_and_skips_spaces() {
    let typeface = Typeface::roboto().unwrap();
    let shaped = typeface.shape("#1 4", TWENTY);
    assert!((shaped.width() - typeface.measure("#1 4", TWENTY)).abs() < 1e-4);
    assert_eq!(shaped.glyphs.len(), 3);
}

#[test]
fn semibold_is_wider_than_regular() {
    let typeface = Typeface::roboto().unwrap();
    let semibold = TextStyle::new(20.0, Weight::SemiBold).unwrap();
    assert!(typeface.measure("#14", semibold) > typeface.measure("#14", TWENTY));
    assert_eq!((semibold.digit_px(), semibold.weight()), (20.0, Weight::SemiBold));
}

#[test]
fn a_size_that_is_not_a_positive_number_is_refused() {
    for digit_px in [0.0, -3.0, f32::NAN, f32::INFINITY] {
        assert!(matches!(
            TextStyle::new(digit_px, Weight::Regular),
            Err(Error::DigitSize(_))
        ));
    }
}

#[test]
fn a_debugged_typeface_does_not_dump_the_font() {
    let typeface = Typeface::roboto().unwrap();
    assert!(format!("{typeface:?}").len() < 100);
}
