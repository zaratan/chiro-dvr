use std::num::NonZeroU32;

use draw::canvas::Canvas;
use draw::color::{INK, PALETTE, Rgb};
use draw::style::style_for;
use draw::text::Typeface;

use super::{Marker, draw_marker};

const GRAY: Rgb = Rgb::new(128, 128, 128);

fn at(canvas: &Canvas, x: u32, y: u32) -> Rgb {
    let index = ((y * canvas.width() + x) * 4) as usize;
    let pixel = &canvas.rgba()[index..index + 3];
    Rgb::new(pixel[0], pixel[1], pixel[2])
}

#[test]
fn marker_is_a_colored_disc_with_a_dark_number() {
    let typeface = Typeface::roboto().unwrap();
    let style = style_for(NonZeroU32::new(1440).unwrap(), "99", &typeface);
    let mut canvas = Canvas::new(100, 100, GRAY).unwrap();
    let marker = Marker {
        center: [50.0, 50.0],
        label: "7",
        color: PALETTE[0],
    };
    draw_marker(&mut canvas, &typeface, marker, &style);
    let inner = (style.radius - 4.0) as u32;
    let disc: Vec<Rgb> = (50 - inner..50 + inner)
        .flat_map(|y| (50 - inner..50 + inner).map(move |x| (x, y)))
        .map(|(x, y)| at(&canvas, x, y))
        .collect();
    assert!(disc.contains(&PALETTE[0]));
    assert!(disc.contains(&INK));
    assert_eq!(at(&canvas, 2, 2), GRAY);
}
