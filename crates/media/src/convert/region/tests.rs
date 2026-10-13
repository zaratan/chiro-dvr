use super::Region;
use crate::convert::{Converter, Error, SourceRange};
use crate::frame::{ColorRange, YuvFrame};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;
const INSIDE: Region = Region {
    left: 10,
    top: 6,
    width: 20,
    height: 14,
};

fn textured() -> YuvFrame {
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    for (index, value) in frame.y_mut().iter_mut().enumerate() {
        *value = (index * 37 % 256) as u8;
    }
    for (index, value) in frame.u_mut().iter_mut().enumerate() {
        *value = (60 + index * 11 % 140) as u8;
    }
    for (index, value) in frame.v_mut().iter_mut().enumerate() {
        *value = (50 + index * 7 % 160) as u8;
    }
    frame
}

fn pixels_of(rgba: &[u8], region: Region) -> Vec<u8> {
    let row = WIDTH as usize * 4;
    (region.top..region.top + region.height)
        .flat_map(|y| {
            let start = y as usize * row + region.left as usize * 4;
            rgba[start..start + region.width as usize * 4].to_vec()
        })
        .collect()
}

#[test]
fn a_region_reads_the_pixels_of_the_whole_conversion_and_leaves_the_rest_alone() {
    let mut converter = Converter::new(WIDTH, HEIGHT, SourceRange::FullBlackAt16).unwrap();
    let frame = textured();
    let mut whole = vec![0; (WIDTH * HEIGHT * 4) as usize];
    converter.yuv_to_rgba(&frame, &mut whole).unwrap();
    let mut part = vec![7; whole.len()];
    converter.yuv_to_rgba_region(&frame, &mut part, INSIDE).unwrap();
    assert_eq!(pixels_of(&part, INSIDE), pixels_of(&whole, INSIDE));
    let changed = part.iter().filter(|value| **value != 7).count();
    assert_eq!(
        changed,
        pixels_of(&whole, INSIDE).iter().filter(|value| **value != 7).count()
    );
}

#[test]
fn a_region_written_back_gives_the_luma_of_the_whole_conversion_and_leaves_the_rest_alone() {
    let mut converter = Converter::new(WIDTH, HEIGHT, SourceRange::FullBlackAt16).unwrap();
    let mut rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
    converter.yuv_to_rgba(&textured(), &mut rgba).unwrap();
    let mut whole = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.rgba_to_yuv(&rgba, &mut whole).unwrap();
    let mut part = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    part.y_mut().fill(3);
    converter.rgba_to_yuv_region(&rgba, &mut part, INSIDE).unwrap();
    for y in 0..HEIGHT as usize {
        for x in 0..WIDTH as usize {
            let index = y * WIDTH as usize + x;
            let inside = (10..30).contains(&x) && (6..20).contains(&y);
            let expected = if inside { whole.y()[index] } else { 3 };
            assert_eq!(part.y()[index], expected, "({x}, {y})");
        }
    }
    let chroma_row = WIDTH as usize / 2;
    for y in 4..9 {
        for x in 6..14 {
            let index = y * chroma_row + x;
            assert_eq!(
                (part.u()[index], part.v()[index]),
                (whole.u()[index], whole.v()[index]),
                "({x}, {y})"
            );
        }
    }
}

#[test]
fn a_region_is_written_back_only_into_a_limited_range_frame() {
    let mut converter = Converter::new(WIDTH, HEIGHT, SourceRange::FullBlackAt16).unwrap();
    let rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
    let mut full = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Full).unwrap();
    assert!(matches!(
        converter.rgba_to_yuv_region(&rgba, &mut full, INSIDE),
        Err(Error::TargetRange {
            found: ColorRange::Full
        })
    ));
}

#[test]
fn on_grey_across_a_full_width_frame_the_display_tables_give_the_round_trip_through_rgba() {
    let mut converter = Converter::new(1440, 2, SourceRange::FullBlackAt16).unwrap();
    let mut frame = YuvFrame::new(1440, 2, ColorRange::Limited).unwrap();
    for (index, value) in frame.y_mut().iter_mut().enumerate() {
        *value = (index * 7 % 256) as u8;
    }
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    let mut rgba = vec![0; 1440 * 2 * 4];
    let mut round_trip = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.yuv_to_rgba(&frame, &mut rgba).unwrap();
    converter.rgba_to_yuv(&rgba, &mut round_trip).unwrap();
    let mut table = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.display(&frame, &mut table).unwrap();
    assert_eq!(table, round_trip);
}

#[test]
fn an_odd_empty_or_overflowing_region_is_refused() {
    let mut converter = Converter::new(WIDTH, HEIGHT, SourceRange::FullBlackAt16).unwrap();
    let frame = textured();
    let mut rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
    for region in [
        Region { left: 1, ..INSIDE },
        Region { width: 0, ..INSIDE },
        Region { left: 50, ..INSIDE },
        Region { top: 40, ..INSIDE },
    ] {
        assert!(matches!(
            converter.yuv_to_rgba_region(&frame, &mut rgba, region),
            Err(Error::Region { .. })
        ));
    }
}

#[test]
fn the_display_tables_send_black_at_16_to_limited_black_white_to_limited_white_and_squeeze_the_colour() {
    let converter = Converter::new(4, 2, SourceRange::FullBlackAt16).unwrap();
    let mut frame = YuvFrame::new(4, 2, ColorRange::Limited).unwrap();
    frame.y_mut().copy_from_slice(&[0, 16, 17, 255, 128, 135, 200, 240]);
    frame.u_mut().copy_from_slice(&[0, 255]);
    frame.v_mut().copy_from_slice(&[128, 100]);
    let mut out = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.display(&frame, &mut out).unwrap();
    assert_eq!((out.y()[0], out.y()[1], out.y()[3]), (16, 16, 235));
    assert_eq!(out.u(), [16, 240]);
    assert_eq!(out.v(), [128, 103]);
    assert_eq!(out.range(), ColorRange::Limited);
}

#[test]
fn on_grey_the_display_tables_give_the_round_trip_through_rgba() {
    let mut converter = Converter::new(256, 2, SourceRange::FullBlackAt16).unwrap();
    let mut frame = YuvFrame::new(256, 2, ColorRange::Limited).unwrap();
    for (index, value) in frame.y_mut().iter_mut().enumerate() {
        *value = (index % 256) as u8;
    }
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    let mut rgba = vec![0; 256 * 2 * 4];
    let mut round_trip = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.yuv_to_rgba(&frame, &mut rgba).unwrap();
    converter.rgba_to_yuv(&rgba, &mut round_trip).unwrap();
    let mut table = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.display(&frame, &mut table).unwrap();
    assert_eq!(table, round_trip);
}

#[test]
fn a_size_evicted_from_the_cache_converts_again_like_the_whole_frame() {
    let mut converter = Converter::new(WIDTH, HEIGHT, SourceRange::FullBlackAt16).unwrap();
    let frame = textured();
    let mut whole = vec![0; (WIDTH * HEIGHT * 4) as usize];
    converter.yuv_to_rgba(&frame, &mut whole).unwrap();
    let mut part = vec![0; whole.len()];
    for step in 0..40 {
        let region = Region {
            left: 0,
            top: 0,
            width: 2 + 2 * (step % 30),
            height: 2 + 2 * (step / 30),
        };
        converter.yuv_to_rgba_region(&frame, &mut part, region).unwrap();
    }
    converter.yuv_to_rgba_region(&frame, &mut part, INSIDE).unwrap();
    assert_eq!(pixels_of(&part, INSIDE), pixels_of(&whole, INSIDE));
}
