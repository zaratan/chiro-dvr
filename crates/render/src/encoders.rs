use std::num::NonZeroU32;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use media::encode::fanout::Fanout;
use media::encode::{self, Codec, Settings, Stamps};
use media::frame::YuvFrame;
use types::rate::FrameRate;
use types::rational::Rational;

use crate::clips::Error;
use crate::zoom::SLOW_MOTION;

#[cfg(test)]
mod tests;

pub(crate) trait Sink: Send {
    fn open(&mut self, path: &Path, frames: usize) -> Result<(), encode::Error>;
    fn open_stamped(&mut self, path: &Path, stamps: Stamps) -> Result<(), encode::Error>;
    fn push(&mut self, frame: &YuvFrame) -> Result<(), encode::Error>;
    fn finish(self) -> Result<(), encode::Error>;
}

impl Sink for Fanout {
    fn open(&mut self, path: &Path, frames: usize) -> Result<(), encode::Error> {
        Fanout::open(self, path, frames)
    }

    fn open_stamped(&mut self, path: &Path, stamps: Stamps) -> Result<(), encode::Error> {
        Fanout::open_stamped(self, path, stamps)
    }

    fn push(&mut self, frame: &YuvFrame) -> Result<(), encode::Error> {
        Fanout::push(self, frame)
    }

    fn finish(self) -> Result<(), encode::Error> {
        Fanout::finish(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Purpose {
    Clips,
    Annotated,
    Zoom,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Request {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rate: Rational,
    pub(crate) codec: Codec,
    pub(crate) key_interval: NonZeroU32,
    pub(crate) keys_on_every_codec: bool,
}

impl Request {
    pub(crate) fn settings(&self) -> Result<Settings, encode::Error> {
        let settings = Settings::new(self.width, self.height, self.rate, self.codec)?;
        Ok(if self.codec == Codec::OpenH264 || self.keys_on_every_codec {
            settings.with_key_interval(self.key_interval)
        } else {
            settings
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

    pub(crate) fn open(
        &self,
        path: &Path,
        width: u32,
        height: u32,
        purpose: Purpose,
    ) -> Result<(O::Sink, Codec), Error> {
        let rate = match purpose {
            Purpose::Zoom => self
                .rate
                .slowed(SLOW_MOTION)
                .map_err(|source| Error::SlowMotion { source })?,
            Purpose::Clips | Purpose::Annotated => self.rate,
        };
        let hardware = self.hardware_allowed();
        let allowed = |codec: Codec| match purpose {
            Purpose::Zoom => codec == Codec::OpenH264,
            Purpose::Clips | Purpose::Annotated => codec != Codec::VideoToolbox || hardware,
        };
        self.open_with(path, (width, height), (rate, purpose == Purpose::Annotated), allowed)
    }

    fn open_with(
        &self,
        path: &Path,
        (width, height): (u32, u32),
        (rate, keys_on_every_codec): (FrameRate, bool),
        allowed: impl Fn(Codec) -> bool,
    ) -> Result<(O::Sink, Codec), Error> {
        let mut refused = None;
        for codec in self.codecs.iter().copied().filter(|codec| allowed(*codec)) {
            let request = Request {
                width,
                height,
                rate: rate.get(),
                codec,
                key_interval: self.key_interval,
                keys_on_every_codec,
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
