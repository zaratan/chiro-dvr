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
