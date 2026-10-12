use std::num::NonZeroU32;
use std::ops::Range;
use std::path::{Path, PathBuf};

use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use types::rational::Rational;

pub(crate) const FRAMES: usize = 48;
const WIDTH: u32 = 1440;
const HEIGHT: u32 = 1080;
const KEY_INTERVAL: u32 = 8;

pub(crate) fn binoculars(folder: &Path) -> PathBuf {
    let path = folder.join("binoculars.mp4");
    let settings = Settings::new(WIDTH, HEIGHT, Rational::new(10_000, 333).unwrap(), Codec::OpenH264)
        .unwrap()
        .with_key_interval(NonZeroU32::new(KEY_INTERVAL).unwrap());
    let mut encoder = Encoder::create(&path, &settings).unwrap();
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    for index in 0..FRAMES {
        for (row, line) in frame.y_mut().chunks_mut(WIDTH as usize).enumerate() {
            for (column, value) in line.iter_mut().enumerate() {
                let texture = ((column + 7 * index) ^ (row + 3 * index)) % 200;
                *value = 16 + u8::try_from(texture).unwrap();
            }
        }
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
    path
}

pub(crate) fn losing_frames(folder: &Path, parts: Range<usize>, count: usize) -> PathBuf {
    let path = binoculars(folder);
    media::payload::wipe(&path, parts, count).unwrap();
    path
}
