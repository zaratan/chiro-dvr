use std::path::{Path, PathBuf};

use types::rational::Rational;

use crate::encode::{Codec, Encoder, Settings};
use crate::frame::{ColorRange, YuvFrame};

pub(crate) const WIDTH: u32 = 320;
pub(crate) const HEIGHT: u32 = 240;

pub(crate) fn pattern(index: u8) -> YuvFrame {
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    for (offset, pixel) in frame.y_mut().iter_mut().enumerate() {
        *pixel = 16_u8
            .wrapping_add(index.wrapping_mul(3))
            .wrapping_add((offset % 200) as u8);
    }
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    frame
}

pub(crate) fn encode(folder: &Path, codec: Codec, frames: u8) -> PathBuf {
    let path = folder.join("sample.mp4");
    let settings = Settings::new(WIDTH, HEIGHT, Rational::new(30, 1).unwrap(), codec).unwrap();
    let mut encoder = Encoder::create(&path, &settings).unwrap();
    for index in 0..frames {
        encoder.push(&pattern(index)).unwrap();
    }
    encoder.finish().unwrap();
    path
}
