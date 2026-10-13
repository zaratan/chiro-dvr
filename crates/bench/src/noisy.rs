use std::path::Path;

use media::encode::{self, Codec, Encoder, Settings};
use media::frame::{self, ColorRange, YuvFrame};
use types::frame::FrameIndex;
use types::rational::{self, Rational};

use crate::random::Xoshiro256;
use crate::synthetic::{self, Class, Motion, Point, Target};

#[cfg(test)]
mod tests;

pub const WIDTH: u32 = 96;
pub const HEIGHT: u32 = 72;
pub const FRAMES: usize = 60;
const NOISE_LOW: f64 = 122.0;
const NOISE_HIGH: f64 = 130.0;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the frame rate of the noisy video is invalid")]
    Rate(#[from] rational::Error),
    #[error("cannot make a frame of the noisy video")]
    Frame(#[from] frame::Error),
    #[error("cannot encode the noisy video")]
    Encode(#[from] encode::Error),
}

pub fn write_video(path: &Path) -> Result<(), Error> {
    let settings = Settings::new(WIDTH, HEIGHT, Rational::new(30, 1)?, Codec::OpenH264)?;
    let mut encoder = Encoder::create(path, &settings)?;
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited)?;
    let mut random = Xoshiro256::seeded(7);
    for _ in 0..FRAMES {
        for pixel in frame.y_mut() {
            *pixel = random.uniform(NOISE_LOW, NOISE_HIGH).round() as u8;
        }
        frame.u_mut().fill(128);
        frame.v_mut().fill(128);
        encoder.push(&frame)?;
    }
    encoder.finish()?;
    Ok(())
}

pub fn slow_target(id: usize, start: usize, frames: usize) -> Result<Target, synthetic::Error> {
    let positions = (0..frames)
        .map(|offset| Point {
            x: 8.0 + 2.0 * offset as f64,
            y: f64::from(HEIGHT) / 2.0,
        })
        .collect();
    let class = Class::new(-60.0, 3.0, Motion::Pass)?;
    Target::new(id, FrameIndex::new(start), positions, vec![-60.0; frames], class)
}
