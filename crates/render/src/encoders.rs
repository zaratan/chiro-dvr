use std::num::NonZeroU32;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use media::encode::{self, Codec, Encoder, Settings};
use media::frame::YuvFrame;
use types::rate::FrameRate;
use types::rational::Rational;

use crate::clips::Error;
use crate::windows::{Group, HARDWARE_LANES};

#[cfg(test)]
mod tests;

pub(crate) trait Sink: Send {
    fn push(&mut self, frame: &YuvFrame) -> Result<(), encode::Error>;
    fn finish(self) -> Result<(), encode::Error>;
}

impl Sink for Encoder {
    fn push(&mut self, frame: &YuvFrame) -> Result<(), encode::Error> {
        Encoder::push(self, frame)
    }

    fn finish(self) -> Result<(), encode::Error> {
        Encoder::finish(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Request<'a> {
    pub(crate) path: &'a Path,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rate: Rational,
    pub(crate) codec: Codec,
    pub(crate) key_interval: NonZeroU32,
}

impl Request<'_> {
    pub(crate) fn settings(&self) -> Result<Settings, encode::Error> {
        let settings = Settings::new(self.width, self.height, self.rate, self.codec)?;
        Ok(match self.codec {
            Codec::OpenH264 => settings.with_key_interval(self.key_interval),
            Codec::VideoToolbox => settings,
        })
    }
}

pub(crate) trait Open: Sync {
    type Sink: Sink;
    fn open(&self, request: &Request<'_>) -> Result<Self::Sink, encode::Error>;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Ffmpeg;

impl Open for Ffmpeg {
    type Sink = Encoder;

    fn open(&self, request: &Request<'_>) -> Result<Encoder, encode::Error> {
        Encoder::create(request.path, &request.settings()?)
    }
}

#[derive(Debug)]
pub(crate) struct Encoders<'a, O> {
    open: &'a O,
    codecs: &'a [Codec],
    rate: FrameRate,
    key_interval: NonZeroU32,
    hardware_failed: AtomicBool,
}

impl<'a, O: Open> Encoders<'a, O> {
    pub(crate) fn new(open: &'a O, codecs: &'a [Codec], rate: FrameRate) -> Self {
        let per_second = rate.get().to_f64().round() as u32;
        Self {
            open,
            codecs,
            rate,
            key_interval: NonZeroU32::new(per_second).unwrap_or(NonZeroU32::MIN),
            hardware_failed: AtomicBool::new(false),
        }
    }

    pub(crate) fn for_lane(&self, lane: usize) -> impl Iterator<Item = Codec> + '_ {
        let hardware = lane < HARDWARE_LANES && !self.hardware_failed.load(Ordering::Acquire);
        self.codecs
            .iter()
            .copied()
            .filter(move |codec| *codec != Codec::VideoToolbox || hardware)
    }

    pub(crate) fn hardware_sessions(&self, group: &Group) -> usize {
        if self.for_lane(0).any(|codec| codec == Codec::VideoToolbox) {
            group.hardware_lanes()
        } else {
            0
        }
    }

    pub(crate) fn set_hardware_aside(&self) {
        self.hardware_failed.store(true, Ordering::Release);
    }

    pub(crate) fn open(&self, path: &Path, lane: usize, width: u32, height: u32) -> Result<(O::Sink, Codec), Error> {
        let mut refused = None;
        for codec in self.for_lane(lane) {
            let request = Request {
                path,
                width,
                height,
                rate: self.rate.get(),
                codec,
                key_interval: self.key_interval,
            };
            match self.open.open(&request) {
                Ok(sink) => return Ok((sink, codec)),
                Err(source) if source.concerns_the_destination() => return Err(Error::writing(path, source)),
                Err(source) => refused = Some(source),
            }
        }
        Err(refused.map_or_else(
            || Error::NoEncoder {
                path: path.to_path_buf(),
            },
            |source| Error::writing(path, source),
        ))
    }
}
