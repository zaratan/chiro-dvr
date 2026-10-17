use super::{BOX, INK, PALETTE, PANEL, PANEL_TEXT, Rgb, SEPARATOR, TRAIL};

const V011_PALETTE_BGR: [(u8, u8, u8); 12] = [
    (10, 214, 255),
    (240, 201, 76),
    (26, 140, 255),
    (91, 227, 91),
    (203, 92, 255),
    (255, 168, 110),
    (60, 240, 200),
    (255, 140, 185),
    (92, 92, 255),
    (235, 235, 235),
    (140, 200, 255),
    (170, 170, 20),
];

fn from_bgr((blue, green, red): (u8, u8, u8)) -> Rgb {
    Rgb::new(red, green, blue)
}

#[test]
fn each_palette_colour_is_the_v011_bgr_colour_reversed() {
    let expected: Vec<Rgb> = V011_PALETTE_BGR.into_iter().map(from_bgr).collect();
    assert_eq!(PALETTE.to_vec(), expected);
}

#[test]
fn the_panel_and_overlay_colours_are_the_v011_bgr_colours_reversed() {
    assert_eq!(INK, from_bgr((0, 0, 0)));
    assert_eq!(PANEL, from_bgr((32, 32, 32)));
    assert_eq!(PANEL_TEXT, from_bgr((230, 230, 230)));
    assert_eq!(SEPARATOR, from_bgr((80, 80, 80)));
    assert_eq!(BOX, from_bgr((0, 0, 255)));
    assert_eq!(TRAIL, from_bgr((0, 200, 255)));
}

const WCAG_AA_TEXT: f64 = 4.5;
const CHANNEL_MAX: f64 = 255.0;
const SRGB_LINEAR_LIMIT: f64 = 0.039_28;
const SRGB_LINEAR_SLOPE: f64 = 12.92;
const SRGB_OFFSET: f64 = 0.055;
const SRGB_GAMMA: f64 = 2.4;
const LUMINANCE_WEIGHTS: [f64; 3] = [0.2126, 0.7152, 0.0722];
const FLARE: f64 = 0.05;

fn relative_luminance(colour: Rgb) -> f64 {
    let linear = |channel: u8| {
        let value = f64::from(channel) / CHANNEL_MAX;
        if value <= SRGB_LINEAR_LIMIT {
            value / SRGB_LINEAR_SLOPE
        } else {
            ((value + SRGB_OFFSET) / (1.0 + SRGB_OFFSET)).powf(SRGB_GAMMA)
        }
    };
    [colour.red, colour.green, colour.blue]
        .into_iter()
        .zip(LUMINANCE_WEIGHTS)
        .map(|(channel, weight)| weight * linear(channel))
        .sum()
}

fn contrast(one: Rgb, other: Rgb) -> f64 {
    let (a, b) = (relative_luminance(one), relative_luminance(other));
    (a.max(b) + FLARE) / (a.min(b) + FLARE)
}

#[test]
fn the_ink_of_a_track_number_reads_on_every_palette_colour_at_the_wcag_aa_ratio_for_text() {
    for colour in PALETTE {
        let ratio = contrast(INK, colour);
        assert!(ratio >= WCAG_AA_TEXT, "{colour:?}: {ratio:.2}");
    }
}
