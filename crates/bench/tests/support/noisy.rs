use std::path::Path;

use bench::random::Xoshiro256;
use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use reference::bench::{TargetLine, write_targets};
use types::rational::Rational;

const WIDTH: u32 = 96;
const HEIGHT: u32 = 72;
const FRAMES: usize = 60;

#[cfg(test)]
pub(crate) fn write_video(path: &Path) {
    let settings = Settings::new(WIDTH, HEIGHT, Rational::new(30, 1).unwrap(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(path, &settings).unwrap();
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    let mut random = Xoshiro256::seeded(7);
    for _ in 0..FRAMES {
        for pixel in frame.y_mut() {
            *pixel = random.uniform(122.0, 130.0).round() as u8;
        }
        frame.u_mut().fill(128);
        frame.v_mut().fill(128);
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
}

#[cfg(test)]
pub(crate) fn write_slow_target(path: &Path) {
    let line = TargetLine {
        id: 0,
        motion: "pass".to_owned(),
        amplitude: -60.0,
        sigma: 3.0,
        start_frame: 10,
        positions: (0..40).map(|offset| [8.0 + 2.0 * f64::from(offset), 36.0]).collect(),
        amplitudes: vec![-60.0; 40],
        effective: Vec::new(),
        patch: None,
    };
    write_targets(path, &[line]).unwrap();
}
