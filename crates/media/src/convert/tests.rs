use super::{Converter, Error, SourceRange};
use crate::frame::{ColorRange, YuvFrame};

const PALETTE: [[u8; 3]; 12] = [
    [255, 214, 10],
    [76, 201, 240],
    [255, 140, 26],
    [91, 227, 91],
    [255, 92, 203],
    [110, 168, 255],
    [200, 240, 60],
    [185, 140, 255],
    [255, 92, 92],
    [235, 235, 235],
    [255, 200, 140],
    [20, 170, 170],
];
const BLOCK: usize = 16;
const COLUMNS: usize = 4;
const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;

fn tiles() -> Vec<u8> {
    let mut rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
    for (index, pixel) in rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let (x, y) = (index % WIDTH as usize, index / WIDTH as usize);
        let [red, green, blue] = PALETTE[(y / BLOCK) * COLUMNS + x / BLOCK];
        *pixel = [red, green, blue, 255];
    }
    rgba
}

fn flat(width: u32, height: u32, y: u8, u: u8, v: u8) -> YuvFrame {
    let mut frame = YuvFrame::new(width, height, ColorRange::Limited).unwrap();
    frame.y_mut().fill(y);
    frame.u_mut().fill(u);
    frame.v_mut().fill(v);
    frame
}

fn first_rgb(converter: &mut Converter, frame: &YuvFrame, width: u32, height: u32) -> [u8; 4] {
    let mut rgba = vec![0; (width * height * 4) as usize];
    converter.yuv_to_rgba(frame, &mut rgba).unwrap();
    [rgba[0], rgba[1], rgba[2], rgba[3]]
}

#[test]
fn palette_tiles_aligned_on_chroma_blocks_come_back_within_two_levels() {
    let mut converter = Converter::new(WIDTH, HEIGHT, SourceRange::Limited).unwrap();
    let source = tiles();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.rgba_to_yuv(&source, &mut frame).unwrap();
    let mut back = vec![0; source.len()];
    converter.yuv_to_rgba(&frame, &mut back).unwrap();
    let worst = source.iter().zip(&back).map(|(a, b)| a.abs_diff(*b)).max().unwrap();
    assert!(worst <= 2, "worst channel difference {worst}");
}

#[test]
fn limited_range_black_and_white_open_to_the_full_rgb_range() {
    let mut converter = Converter::new(4, 2, SourceRange::Limited).unwrap();
    assert_eq!(
        first_rgb(&mut converter, &flat(4, 2, 16, 128, 128), 4, 2),
        [0, 0, 0, 255]
    );
    assert_eq!(
        first_rgb(&mut converter, &flat(4, 2, 235, 128, 128), 4, 2),
        [255, 255, 255, 255]
    );
}

#[test]
fn pure_red_takes_the_bt709_luma_not_the_bt601_one() {
    let mut converter = Converter::new(2, 2, SourceRange::Limited).unwrap();
    let red = [255, 0, 0, 255].repeat(4);
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.rgba_to_yuv(&red, &mut frame).unwrap();
    assert_eq!((frame.y()[0], frame.u()[0], frame.v()[0]), (63, 102, 240));
    assert_eq!(frame.range(), ColorRange::Limited);
}

#[test]
fn an_odd_size_converts_both_ways() {
    let mut converter = Converter::new(33, 17, SourceRange::Limited).unwrap();
    let gray = [128, 128, 128, 255].repeat(33 * 17);
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.rgba_to_yuv(&gray, &mut frame).unwrap();
    assert_eq!((frame.width(), frame.height()), (33, 17));
    let mut back = vec![0; gray.len()];
    converter.yuv_to_rgba(&frame, &mut back).unwrap();
    assert!(back.iter().zip(&gray).all(|(a, b)| a.abs_diff(*b) <= 1));
}

#[test]
fn the_stated_source_range_wins_over_the_label_of_the_frame() {
    let mut limited = Converter::new(2, 2, SourceRange::Limited).unwrap();
    let mut full = Converter::new(2, 2, SourceRange::Full).unwrap();
    for label in [ColorRange::Limited, ColorRange::Full, ColorRange::Unspecified] {
        let mut frame = YuvFrame::new(2, 2, label).unwrap();
        frame.y_mut().fill(16);
        frame.u_mut().fill(128);
        frame.v_mut().fill(128);
        assert_eq!(first_rgb(&mut limited, &frame, 2, 2), [0, 0, 0, 255]);
        assert_eq!(first_rgb(&mut full, &frame, 2, 2), [16, 16, 16, 255]);
    }
}

#[test]
fn a_full_range_source_keeps_its_darkest_and_brightest_values_apart() {
    let mut converter = Converter::new(4, 2, SourceRange::Full).unwrap();
    assert_eq!(
        first_rgb(&mut converter, &flat(4, 2, 0, 128, 128), 4, 2),
        [0, 0, 0, 255]
    );
    assert_eq!(
        first_rgb(&mut converter, &flat(4, 2, 8, 128, 128), 4, 2),
        [8, 8, 8, 255]
    );
    assert_eq!(
        first_rgb(&mut converter, &flat(4, 2, 245, 128, 128), 4, 2),
        [245, 245, 245, 255]
    );
    assert_eq!(
        first_rgb(&mut converter, &flat(4, 2, 255, 128, 128), 4, 2),
        [255, 255, 255, 255]
    );
}

#[test]
fn a_full_range_source_is_written_back_squeezed_into_the_limited_range() {
    let mut converter = Converter::new(2, 2, SourceRange::Full).unwrap();
    let mut rgba = vec![0; 16];
    converter.yuv_to_rgba(&flat(2, 2, 0, 128, 128), &mut rgba).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.rgba_to_yuv(&rgba, &mut frame).unwrap();
    assert_eq!((frame.y()[0], frame.range()), (16, ColorRange::Limited));
    converter.yuv_to_rgba(&flat(2, 2, 255, 128, 128), &mut rgba).unwrap();
    converter.rgba_to_yuv(&rgba, &mut frame).unwrap();
    assert_eq!(frame.y()[0], 235);
}

#[test]
fn a_frame_of_another_size_is_refused() {
    let mut converter = Converter::new(4, 4, SourceRange::Limited).unwrap();
    let frame = flat(2, 2, 16, 128, 128);
    let mut rgba = vec![0; 64];
    assert!(matches!(
        converter.yuv_to_rgba(&frame, &mut rgba),
        Err(Error::FrameSize { found_width: 2, .. })
    ));
}

#[test]
fn a_buffer_of_the_wrong_length_is_refused_both_ways() {
    let mut converter = Converter::new(2, 2, SourceRange::Limited).unwrap();
    let mut short = vec![0; 15];
    assert!(matches!(
        converter.yuv_to_rgba(&flat(2, 2, 16, 128, 128), &mut short),
        Err(Error::BufferSize {
            expected: 16,
            found: 15
        })
    ));
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    assert!(matches!(
        converter.rgba_to_yuv(&short, &mut frame),
        Err(Error::BufferSize { expected: 16, .. })
    ));
}

#[test]
fn an_empty_size_is_refused() {
    assert!(matches!(
        Converter::new(0, 4, SourceRange::Limited),
        Err(Error::Size { width: 0, height: 4 })
    ));
}

#[test]
fn a_debugged_converter_shows_its_size_not_ffmpeg_internals() {
    let converter = Converter::new(6, 4, SourceRange::Limited).unwrap();
    let text = format!("{converter:?}");
    assert!(text.contains("width: 6") && text.contains("height: 4"), "{text}");
}

#[test]
fn grayish_video_left_undrawn_comes_back_close_to_itself() {
    let mut converter = Converter::new(WIDTH, HEIGHT, SourceRange::Limited).unwrap();
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    for (pixel, ramp) in frame.y_mut().iter_mut().zip((32..220_u8).cycle()) {
        *pixel = ramp;
    }
    for (u, index) in frame.u_mut().iter_mut().zip((0..=u8::MAX).cycle()) {
        *u = 120 + index % 16;
    }
    for (v, index) in frame.v_mut().iter_mut().zip((0..=u8::MAX).cycle()) {
        *v = 136 - index % 16;
    }
    let original = frame.clone();
    let mut rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
    converter.yuv_to_rgba(&frame, &mut rgba).unwrap();
    converter.rgba_to_yuv(&rgba, &mut frame).unwrap();
    let worst = |a: &[u8], b: &[u8]| a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap();
    let errors = (
        worst(original.y(), frame.y()),
        worst(original.u(), frame.u()),
        worst(original.v(), frame.v()),
    );
    assert!(errors.0 <= 2 && errors.1 <= 2 && errors.2 <= 2, "{errors:?}");
}

#[test]
fn every_converted_pixel_is_opaque_in_either_source_range() {
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    for (index, level) in frame.y_mut().iter_mut().enumerate() {
        *level = (index % 256) as u8;
    }
    for (index, level) in frame.u_mut().iter_mut().enumerate() {
        *level = (index * 7 % 256) as u8;
    }
    for (index, level) in frame.v_mut().iter_mut().enumerate() {
        *level = (index * 13 % 256) as u8;
    }
    for range in [SourceRange::Full, SourceRange::Limited] {
        let mut converter = Converter::new(WIDTH, HEIGHT, range).unwrap();
        let mut rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
        converter.yuv_to_rgba(&frame, &mut rgba).unwrap();
        assert!(rgba.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 255), "{range:?}");
    }
}
