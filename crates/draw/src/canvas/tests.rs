use super::{Canvas, Error, Outline, PixelBox, Point, Segment};
use crate::color::{BOX, INK, PALETTE, Rgb, TRAIL};
use crate::style::TRACK_LABEL;
use crate::text::{TextStyle, Typeface, Weight};

const GRAY: Rgb = Rgb::new(128, 128, 128);
const BLACK: Rgb = Rgb::new(0, 0, 0);
const WHITE: Rgb = Rgb::new(255, 255, 255);

fn at(canvas: &Canvas, x: u32, y: u32) -> Rgb {
    let index = ((y * canvas.width() + x) * 4) as usize;
    let pixel = &canvas.rgba()[index..index + 4];
    Rgb::new(pixel[0], pixel[1], pixel[2])
}

fn lit_rows(canvas: &Canvas) -> Vec<u32> {
    (0..canvas.height())
        .filter(|&y| (0..canvas.width()).any(|x| at(canvas, x, y).red > 127))
        .collect()
}

fn written(text: &str, style: TextStyle) -> Canvas {
    let typeface = Typeface::roboto().unwrap();
    let mut canvas = Canvas::new(120, 60, BLACK).unwrap();
    canvas.text(&typeface, text, Point { x: 10.0, y: 40.0 }, style, WHITE);
    canvas
}

fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

const TWENTY: TextStyle = TextStyle::trusted(20.0, Weight::Regular);
const AREA: PixelBox = PixelBox {
    left: 10,
    top: 10,
    right: 30,
    bottom: 20,
};

fn row(canvas: &Canvas, y: u32, xs: std::ops::RangeInclusive<u32>) -> Vec<Rgb> {
    xs.map(|x| at(canvas, x, y)).collect()
}

fn column(canvas: &Canvas, x: u32, ys: std::ops::RangeInclusive<u32>) -> Vec<Rgb> {
    ys.map(|y| at(canvas, x, y)).collect()
}

#[test]
fn an_empty_canvas_is_refused() {
    assert!(matches!(
        Canvas::new(0, 10, BLACK),
        Err(Error::Size { width: 0, height: 10 })
    ));
}

#[test]
fn a_canvas_starts_opaque_in_its_background_colour() {
    let canvas = Canvas::new(3, 2, GRAY).unwrap();
    assert!(
        canvas
            .rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [128, 128, 128, 255])
    );
}

#[test]
fn an_accented_letter_lights_pixels_above_the_plain_letter() {
    let plain = lit_rows(&written("e", TWENTY));
    let accented = lit_rows(&written("é", TWENTY));
    assert_eq!(plain.last(), accented.last());
    assert!(accented[0] + 3 < plain[0], "{accented:?} {plain:?}");
}

#[test]
fn a_decomposed_accent_draws_the_same_pixels_as_the_precomposed_letter() {
    let composed = written("détections", TWENTY);
    let decomposed = written("de\u{301}tections", TWENTY);
    assert_eq!(composed, decomposed);
}

#[test]
fn a_digit_drawn_at_20_pixels_spans_20_rows() {
    assert_eq!(lit_rows(&written("0", TWENTY)).len(), 20);
}

#[test]
fn the_track_number_digits_are_17_pixels_tall() {
    let rows = lit_rows(&written("2", TRACK_LABEL));
    assert_eq!(rows.len(), 17, "{rows:?}");
}

#[test]
fn a_one_pixel_box_covers_exactly_its_border_pixels_on_all_four_sides() {
    let mut canvas = Canvas::new(40, 30, GRAY).unwrap();
    canvas.rectangle(AREA, BOX, 1);
    assert_eq!(column(&canvas, 20, 9..=11), [GRAY, BOX, GRAY]);
    assert_eq!(column(&canvas, 20, 19..=21), [GRAY, BOX, GRAY]);
    assert_eq!(row(&canvas, 15, 9..=11), [GRAY, BOX, GRAY]);
    assert_eq!(row(&canvas, 15, 29..=31), [GRAY, BOX, GRAY]);
    assert_eq!(at(&canvas, 10, 10), BOX);
}

#[test]
fn a_two_pixel_box_grows_outward_so_it_keeps_the_inner_edge_of_the_one_pixel_box() {
    let mut canvas = Canvas::new(40, 30, GRAY).unwrap();
    canvas.rectangle(AREA, BOX, 2);
    assert_eq!(column(&canvas, 20, 8..=11), [GRAY, BOX, BOX, GRAY]);
    assert_eq!(column(&canvas, 20, 19..=22), [GRAY, BOX, BOX, GRAY]);
    assert_eq!(row(&canvas, 15, 8..=11), [GRAY, BOX, BOX, GRAY]);
    assert_eq!(row(&canvas, 15, 29..=32), [GRAY, BOX, BOX, GRAY]);
    assert_eq!(at(&canvas, 9, 9), BOX);
}

#[test]
fn a_diagonal_line_is_smoothed() {
    let mut canvas = Canvas::new(40, 40, BLACK).unwrap();
    canvas.line(point(2.0, 3.0), point(37.0, 30.0), WHITE, 1.0);
    let partial = canvas
        .rgba()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[0] > 0 && pixel[0] < 255)
        .count();
    assert!(partial > 20, "{partial}");
}

#[test]
fn a_trail_is_drawn_through_every_point() {
    let mut canvas = Canvas::new(60, 40, BLACK).unwrap();
    canvas.polyline(&[point(5.0, 5.0), point(30.0, 5.0), point(30.0, 30.0)], TRAIL, 2.0);
    assert_eq!(at(&canvas, 15, 5), TRAIL);
    assert_eq!(at(&canvas, 30, 20), TRAIL);
    assert_eq!(at(&canvas, 15, 20), BLACK);
}

#[test]
fn each_segment_takes_its_own_width() {
    let mut canvas = Canvas::new(100, 40, BLACK).unwrap();
    let narrow = Segment {
        from: point(5.0, 20.0),
        to: point(50.0, 20.0),
        width: 2.0,
    };
    let wide = Segment {
        from: point(50.0, 20.0),
        to: point(95.0, 20.0),
        width: 8.0,
    };
    canvas.segments(&[narrow, wide], WHITE);
    let height_at = |x| (0..40).filter(|&y| at(&canvas, x, y).red > 127).count();
    assert_eq!(height_at(25), 2);
    assert_eq!(height_at(75), 8);
}

#[test]
fn a_marker_disk_is_filled_and_ringed() {
    let mut canvas = Canvas::new(60, 60, GRAY).unwrap();
    let ring = Outline { color: INK, width: 2.0 };
    canvas.disk(point(30.0, 30.0), 20.0, PALETTE[0], Some(ring));
    assert_eq!(at(&canvas, 30, 30), PALETTE[0]);
    assert_eq!(at(&canvas, 30, 10), INK);
    assert_eq!(at(&canvas, 30, 5), GRAY);
}

#[test]
fn a_polygon_can_be_filled_outlined_or_both() {
    let triangle = [point(10.0, 10.0), point(50.0, 10.0), point(30.0, 45.0)];
    let mut filled = Canvas::new(60, 60, GRAY).unwrap();
    filled.polygon(&triangle, Some(PALETTE[3]), None);
    assert_eq!(at(&filled, 30, 20), PALETTE[3]);
    let mut outlined = Canvas::new(60, 60, GRAY).unwrap();
    outlined.polygon(&triangle, None, Some(Outline { color: INK, width: 2.0 }));
    assert_eq!(at(&outlined, 30, 20), GRAY);
    assert_eq!(at(&outlined, 30, 10), INK);
}

#[test]
fn degenerate_shapes_draw_nothing_rather_than_failing() {
    let mut canvas = Canvas::new(20, 20, GRAY).unwrap();
    let before = canvas.clone();
    canvas.polyline(&[point(3.0, 3.0)], WHITE, 2.0);
    canvas.polyline(&[], WHITE, 2.0);
    canvas.polygon(&[], Some(WHITE), None);
    canvas.disk(point(5.0, 5.0), -1.0, WHITE, None);
    canvas.line(point(1.0, 1.0), point(15.0, 15.0), WHITE, 0.0);
    canvas.line(point(1.0, 1.0), point(15.0, 15.0), WHITE, f32::NAN);
    canvas.rectangle(AREA, WHITE, 0);
    canvas.rectangle(
        PixelBox {
            left: 5,
            top: 5,
            right: 2,
            bottom: 2,
        },
        WHITE,
        1,
    );
    assert_eq!(canvas, before);
}

#[test]
fn dimming_keeps_six_tenths_of_each_channel_and_the_opacity() {
    let mut canvas = Canvas::new(2, 1, Rgb::new(200, 255, 1)).unwrap();
    canvas.dim();
    assert_eq!(&canvas.rgba()[..4], [120, 153, 1, 255]);
}

#[test]
fn everything_drawn_keeps_the_canvas_opaque() {
    let typeface = Typeface::roboto().unwrap();
    let mut canvas = Canvas::new(80, 60, GRAY).unwrap();
    canvas.text(&typeface, "#14 é", point(2.0, 30.0), TRACK_LABEL, BOX);
    canvas.disk(
        point(40.0, 40.0),
        15.3,
        PALETTE[5],
        Some(Outline { color: INK, width: 1.5 }),
    );
    canvas.line(point(0.0, 0.0), point(79.0, 59.0), TRAIL, 1.0);
    canvas.dim();
    assert!(canvas.rgba().as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 255));
}

#[test]
fn written_bytes_are_seen_by_the_canvas() {
    let mut canvas = Canvas::new(2, 2, BLACK).unwrap();
    canvas.rgba_mut()[4..8].copy_from_slice(&[9, 8, 7, 255]);
    assert_eq!(at(&canvas, 1, 0), Rgb::new(9, 8, 7));
    assert_eq!((canvas.width(), canvas.height()), (2, 2));
}
