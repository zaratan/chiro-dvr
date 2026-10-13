use std::num::NonZeroU32;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use media::encode::fanout::Fanout;
use media::encode::{self, Codec, Settings};
use media::frame::YuvFrame;
use types::rate::FrameRate;
use types::rational::Rational;

use crate::clips::Error;

#[cfg(test)]
mod tests;

pub(crate) trait Sink: Send {
    fn open(&mut self, path: &Path, frames: usize) -> Result<(), encode::Error>;
    fn push(&mut self, frame: &YuvFrame) -> Result<(), encode::Error>;
    fn finish(self) -> Result<(), encode::Error>;
}

impl Sink for Fanout {
    fn open(&mut self, path: &Path, frames: usize) -> Result<(), encode::Error> {
        Fanout::open(self, path, frames)
    }

    fn push(&mut self, frame: &YuvFrame) -> Result<(), encode::Error> {
        Fanout::push(self, frame)
    }

    fn finish(self) -> Result<(), encode::Error> {
        Fanout::finish(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Request {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rate: Rational,
    pub(crate) codec: Codec,
    pub(crate) key_interval: NonZeroU32,
}

impl Request {
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
    fn open(&self, request: &Request) -> Result<Self::Sink, encode::Error>;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Ffmpeg;

impl Open for Ffmpeg {
    type Sink = Fanout;

    fn open(&self, request: &Request) -> Result<Fanout, encode::Error> {
        Fanout::create(&request.settings()?)
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

    pub(crate) fn hardware_allowed(&self) -> bool {
        !self.hardware_failed.load(Ordering::Acquire) && self.codecs.contains(&Codec::VideoToolbox)
    }

    pub(crate) fn hardware_sessions(&self) -> usize {
        usize::from(self.hardware_allowed())
    }

    pub(crate) fn set_hardware_aside(&self) {
        self.hardware_failed.store(true, Ordering::Release);
    }

    pub(crate) fn open(&self, path: &Path, width: u32, height: u32) -> Result<(O::Sink, Codec), Error> {
        let mut refused = None;
        let hardware = self.hardware_allowed();
        let usable = self
            .codecs
            .iter()
            .copied()
            .filter(|codec| *codec != Codec::VideoToolbox || hardware);
        for codec in usable {
            let request = Request {
                width,
                height,
                rate: self.rate.get(),
                codec,
                key_interval: self.key_interval,
            };
            match self.open.open(&request) {
                Ok(sink) => return Ok((sink, codec)),
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
