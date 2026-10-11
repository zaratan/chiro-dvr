use media::convert::{Converter, SourceRange};
use media::decode::{Decoder, Threads};
use media::frame::{ColorRange, YuvFrame};

const FRAME: usize = 716;
const MOST_LEVELS: u8 = 2;
const LARGEST_SHARE_BEYOND: f64 = 0.001;
const LARGEST_MEAN_ERROR: f64 = 0.15;

#[cfg(test)]
fn frame_716() -> YuvFrame {
    let path = reference::fixture::fixture("video_092_original_3m24-4m05.mp4").unwrap();
    let mut decoder = Decoder::open(&path, Threads::Automatic).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    for _ in 0..=FRAME {
        decoder.next(&mut frame).unwrap().unwrap();
    }
    frame
}

#[cfg(test)]
fn rgba(frame: &YuvFrame, source: SourceRange) -> Vec<u8> {
    let mut converter = Converter::new(frame.width(), frame.height(), source).unwrap();
    let mut rgba = vec![0; (frame.width() * frame.height() * 4) as usize];
    converter.yuv_to_rgba(frame, &mut rgba).unwrap();
    rgba
}

#[test]
#[ignore = "slow"]
fn read_as_full_range_the_extract_is_pure_black_or_white_only_where_the_source_is() {
    let frame = frame_716();
    let rgba = rgba(&frame, SourceRange::Full);
    for (pixel, y) in rgba.as_chunks::<4>().0.iter().zip(frame.y()) {
        if pixel[..3] == [0, 0, 0] {
            assert_eq!(*y, 0);
        }
        if pixel[..3] == [255, 255, 255] {
            assert_eq!(*y, 255);
        }
    }
}

#[test]
#[ignore = "slow"]
fn an_undrawn_frame_written_in_limited_range_displays_like_the_full_range_source() {
    let frame = frame_716();
    let full = rgba(&frame, SourceRange::Full);
    let mut converter = Converter::new(frame.width(), frame.height(), SourceRange::Limited).unwrap();
    let mut written = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    converter.rgba_to_yuv(&full, &mut written).unwrap();
    assert_eq!(written.range(), ColorRange::Limited);
    let mut shown = vec![0; full.len()];
    converter.yuv_to_rgba(&written, &mut shown).unwrap();
    let errors = || full.iter().zip(&shown).map(|(a, b)| a.abs_diff(*b));
    let values = full.len() as f64;
    let share_beyond = errors().filter(|error| *error > MOST_LEVELS).count() as f64 / values;
    let mean = errors().map(f64::from).sum::<f64>() / values;
    assert!(
        share_beyond <= LARGEST_SHARE_BEYOND && mean <= LARGEST_MEAN_ERROR,
        "{share_beyond} {mean}"
    );
}
