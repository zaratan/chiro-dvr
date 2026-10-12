use media::convert::{Converter, SourceRange};
use media::decode::{Decoder, Threads};
use media::frame::{ColorRange, YuvFrame};

const FRAME: usize = 716;
const MOST_LEVELS: u8 = 2;
const LARGEST_SHARE_BEYOND: f64 = 0.001;
const LARGEST_MEAN_ERROR: f64 = 0.15;
const LARGEST_SHARE_DIFFERING: f64 = 0.002;

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

#[test]
#[ignore = "slow"]
fn the_display_tables_stay_close_to_the_round_trip_through_rgba_on_every_frame_of_the_extract() {
    let path = reference::fixture::fixture("video_092_original_3m24-4m05.mp4").unwrap();
    let mut decoder = Decoder::open(&path, Threads::Automatic).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut converter: Option<(Converter, Vec<u8>)> = None;
    let (mut round_trip, mut table) = (
        YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap(),
        YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap(),
    );
    let (mut largest, mut differing, mut values, mut frames) = ([0_u8; 3], [0_usize; 3], [0_usize; 3], 0);
    let mut beyond = [0_usize; 3];
    while decoder.next(&mut frame).unwrap().is_some() {
        let (converter, rgba) = converter.get_or_insert_with(|| {
            let converter = Converter::new(frame.width(), frame.height(), SourceRange::FullBlackAt16).unwrap();
            (converter, vec![0; frame.width() as usize * frame.height() as usize * 4])
        });
        converter.yuv_to_rgba(&frame, rgba).unwrap();
        converter.rgba_to_yuv(rgba, &mut round_trip).unwrap();
        converter.display(&frame, &mut table).unwrap();
        let planes = [
            (round_trip.y(), table.y()),
            (round_trip.u(), table.u()),
            (round_trip.v(), table.v()),
        ];
        for (plane, (expected, found)) in planes.into_iter().enumerate() {
            for (a, b) in expected.iter().zip(found) {
                let gap = a.abs_diff(*b);
                largest[plane] = largest[plane].max(gap);
                differing[plane] += usize::from(gap > 0);
                beyond[plane] += usize::from(gap > MOST_LEVELS);
            }
            values[plane] += expected.len();
        }
        frames += 1;
    }
    let shares: Vec<f64> = differing
        .iter()
        .zip(values)
        .map(|(count, total)| *count as f64 / total as f64)
        .collect();
    let beyond_shares: Vec<f64> = beyond
        .iter()
        .zip(values)
        .map(|(count, total)| *count as f64 / total as f64)
        .collect();
    println!(
        "{frames} frames: largest gap per plane {largest:?}, share of values that differ {shares:?}, beyond {MOST_LEVELS} levels {beyond_shares:?}"
    );
    assert_eq!(frames, 1230);
    assert!(
        shares.iter().all(|share| *share <= LARGEST_SHARE_DIFFERING),
        "{shares:?}"
    );
    assert!(
        beyond_shares.iter().all(|share| *share <= LARGEST_SHARE_BEYOND),
        "{beyond_shares:?}"
    );
}
