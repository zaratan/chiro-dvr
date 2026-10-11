use std::fs;
use std::path::{Path, PathBuf};

use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use types::rational::Rational;

pub(crate) const WIDTH: u32 = 320;
pub(crate) const HEIGHT: u32 = 240;
pub(crate) const FRAMES: u8 = 30;

pub(crate) fn encode(folder: &Path, frames: u8) -> PathBuf {
    let path = folder.join("sample.mp4");
    let settings = Settings::new(WIDTH, HEIGHT, Rational::new(30, 1).unwrap(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(&path, &settings).unwrap();
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    for index in 0..frames {
        for (pixel, ramp) in frame.y_mut().iter_mut().zip((0..200_u8).cycle()) {
            *pixel = 16_u8.wrapping_add(index.wrapping_mul(3)).wrapping_add(ramp);
        }
        frame.u_mut().fill(128);
        frame.v_mut().fill(128);
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
    path
}

pub(crate) fn binoculars(path: &Path, rate: Rational, planes: impl IntoIterator<Item = Vec<u8>>) -> PathBuf {
    let settings = Settings::new(1440, 1080, rate, Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(path, &settings).unwrap();
    let mut frame = YuvFrame::new(1440, 1080, ColorRange::Limited).unwrap();
    for plane in planes {
        frame.y_mut().copy_from_slice(&plane);
        frame.u_mut().fill(128);
        frame.v_mut().fill(128);
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
    path.to_path_buf()
}

pub(crate) fn damaged(folder: &Path, from_quarter: usize, to_quarter: usize) -> PathBuf {
    let path = encode(folder, FRAMES);
    let mut bytes = fs::read(&path).unwrap();
    let payload = bytes.windows(4).position(|window| window == b"mdat").unwrap() + 4;
    let size: [u8; 4] = bytes[payload - 8..payload - 4].try_into().unwrap();
    let quarter = (usize::try_from(u32::from_be_bytes(size)).unwrap() - 8) / 4;
    bytes[payload + from_quarter * quarter..payload + to_quarter * quarter].fill(0xff);
    fs::write(&path, bytes).unwrap();
    path
}
