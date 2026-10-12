use std::path::Path;

use types::rational::Rational;

use super::{Codec, Encoder, Settings};
use crate::frame::{self, ColorRange, YuvFrame};

#[cfg(test)]
mod tests;

const TRIAL_FRAMES: usize = 8;
const MID_GRAY: u8 = 128;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the fallback lists no encoder")]
    Empty,
    #[error("{0} appears twice in the fallback")]
    Repeated(Codec),
    #[error("the clip settings are refused")]
    Settings(#[source] super::Error),
    #[error("cannot make the trial frame")]
    Frame(#[source] frame::Error),
    #[error("cannot make a folder for the trial")]
    Scratch(#[source] std::io::Error),
    #[error("the clip cannot be written there, whatever the encoder")]
    Destination(#[source] super::Error),
    #[error("no encoder works: {}", describe(failures))]
    NoEncoder { failures: Vec<(Codec, super::Error)> },
}

fn describe(failures: &[(Codec, super::Error)]) -> String {
    let lines: Vec<String> = failures
        .iter()
        .map(|(codec, error)| {
            let causes = std::iter::successors(Some(error as &dyn std::error::Error), |cause| cause.source());
            std::iter::once(codec.to_string())
                .chain(causes.map(ToString::to_string))
                .collect::<Vec<_>>()
                .join(": ")
        })
        .collect();
    lines.join("; ")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fallback {
    first: Codec,
    rest: Vec<Codec>,
}

impl Fallback {
    pub fn new(codecs: Vec<Codec>) -> Result<Self, Error> {
        let mut codecs = codecs.into_iter();
        let first = codecs.next().ok_or(Error::Empty)?;
        let rest: Vec<Codec> = codecs.collect();
        for (index, codec) in rest.iter().enumerate() {
            if *codec == first || rest[..index].contains(codec) {
                return Err(Error::Repeated(*codec));
            }
        }
        Ok(Self { first, rest })
    }

    pub fn for_this_platform() -> Self {
        let (first, rest) = if cfg!(target_os = "macos") {
            (Codec::VideoToolbox, vec![Codec::OpenH264])
        } else {
            (Codec::OpenH264, Vec::new())
        };
        Self { first, rest }
    }

    pub fn codecs(&self) -> impl Iterator<Item = Codec> + '_ {
        std::iter::once(self.first).chain(self.rest.iter().copied())
    }

    pub fn trial(mut self, width: u32, height: u32, frame_rate: Rational) -> Result<Self, Error> {
        let settings = self.settings(width, height, frame_rate)?;
        let mut gray = YuvFrame::new(width, height, ColorRange::Limited).map_err(Error::Frame)?;
        gray.y_mut().fill(MID_GRAY);
        gray.u_mut().fill(MID_GRAY);
        gray.v_mut().fill(MID_GRAY);
        let scratch = tempfile::tempdir().map_err(Error::Scratch)?;
        let (index, first, ()) = first_working(self.codecs(), |index, codec| {
            let path = scratch.path().join(format!("trial-{index}.mp4"));
            encode(&path, &settings.with_codec(codec), &gray)
        })?;
        Ok(Self {
            first,
            rest: self.rest.split_off(index),
        })
    }

    pub fn create(&self, path: &Path, width: u32, height: u32, frame_rate: Rational) -> Result<Encoder, Error> {
        let settings = self.settings(width, height, frame_rate)?;
        let (_, _, encoder) = first_working(self.codecs(), |_, codec| {
            Encoder::create(path, &settings.with_codec(codec))
        })?;
        Ok(encoder)
    }

    fn settings(&self, width: u32, height: u32, frame_rate: Rational) -> Result<Settings, Error> {
        Settings::new(width, height, frame_rate, self.first).map_err(Error::Settings)
    }
}

fn encode(path: &Path, settings: &Settings, frame: &YuvFrame) -> Result<(), super::Error> {
    let mut encoder = Encoder::create(path, settings)?;
    for _ in 0..TRIAL_FRAMES {
        encoder.push(frame)?;
    }
    encoder.finish()
}

fn first_working<T>(
    codecs: impl Iterator<Item = Codec>,
    mut attempt: impl FnMut(usize, Codec) -> Result<T, super::Error>,
) -> Result<(usize, Codec, T), Error> {
    let mut failures = Vec::new();
    for (index, codec) in codecs.enumerate() {
        match attempt(index, codec) {
            Ok(done) => return Ok((index, codec, done)),
            Err(error) if error.concerns_the_destination() => return Err(Error::Destination(error)),
            Err(error) => failures.push((codec, error)),
        }
    }
    Err(Error::NoEncoder { failures })
}
