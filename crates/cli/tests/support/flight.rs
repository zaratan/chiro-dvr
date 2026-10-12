use std::path::Path;

use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use types::rational::Rational;

const WIDTH: usize = 1440;
const HEIGHT: usize = 1080;
const FRAMES: usize = 20;
const RATE: i32 = 10;
const SIDE: usize = 9;
const STEP: usize = 9;
const NIGHT: u8 = 40;
const BAT: u8 = 140;

#[cfg(test)]
pub(crate) fn write_flight(path: &Path) {
    let settings = Settings::new(
        WIDTH as u32,
        HEIGHT as u32,
        Rational::new(RATE, 1).unwrap(),
        Codec::OpenH264,
    )
    .unwrap();
    let mut encoder = Encoder::create(path, &settings).unwrap();
    let mut frame = YuvFrame::new(WIDTH as u32, HEIGHT as u32, ColorRange::Limited).unwrap();
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    for index in 0..FRAMES {
        let left = 20 + STEP * index;
        for (row, line) in frame.y_mut().as_chunks_mut::<WIDTH>().0.iter_mut().enumerate() {
            for (column, value) in line.iter_mut().enumerate() {
                let inside = (left..left + SIDE).contains(&column) && (537..537 + SIDE).contains(&row);
                *value = if inside { BAT } else { NIGHT };
            }
        }
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
}
