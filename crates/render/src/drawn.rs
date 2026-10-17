use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use draw::color::Rgb;
use media::encode::{self, Codec, Stamps};
use media::fingerprint;
use media::frame::YuvFrame;
use types::track::Track;

use crate::clips::{Error, Outputs, Video, render_with};
use crate::config::RenderConfig;
use crate::encoders::{Open, Request, Sink};
use crate::source::Source;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawn {
    pub path: PathBuf,
    pub fingerprints: Vec<u64>,
}

/// The frames `clips::render` would hand to its encoders, as fingerprints, without encoding them.
pub fn drawn(
    source: &Source,
    tracks: &[Track],
    colors: &[Option<Rgb>],
    outputs: &Outputs,
    config: RenderConfig,
) -> Result<Vec<Drawn>, Error> {
    let recorder = Recorder::default();
    let video = Video {
        source,
        tracks,
        colors,
        config,
    };
    let open = &recorder;
    render_with(&open, 1, &video, outputs, &[Codec::OpenH264])?;
    let mut drawn = recorder.drawn.into_inner().unwrap_or_else(PoisonError::into_inner);
    drawn.sort_by(|first, second| first.path.cmp(&second.path));
    Ok(drawn)
}

#[derive(Debug, Default)]
struct Recorder {
    drawn: Mutex<Vec<Drawn>>,
}

#[derive(Debug)]
struct Recording<'a> {
    recorder: &'a Recorder,
    open: Vec<(Drawn, usize)>,
}

impl<'a> Open for &'a Recorder {
    type Sink = Recording<'a>;

    fn open(&self, _: &Request) -> Result<Recording<'a>, encode::Error> {
        Ok(Recording {
            recorder: self,
            open: Vec::new(),
        })
    }
}

impl Sink for Recording<'_> {
    fn open(&mut self, path: &Path, frames: usize) -> Result<(), encode::Error> {
        let drawn = Drawn {
            path: path.to_path_buf(),
            fingerprints: Vec::with_capacity(frames),
        };
        self.open.push((drawn, frames));
        Ok(())
    }

    fn open_stamped(&mut self, path: &Path, stamps: Stamps) -> Result<(), encode::Error> {
        Sink::open(self, path, stamps.frame_count())
    }

    fn push(&mut self, frame: &YuvFrame) -> Result<(), encode::Error> {
        let fingerprint = fingerprint::of(frame);
        for (drawn, frames) in &mut self.open {
            if drawn.fingerprints.len() < *frames {
                drawn.fingerprints.push(fingerprint);
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<(), encode::Error> {
        let finished = self.open.into_iter().map(|(drawn, _)| drawn);
        self.recorder
            .drawn
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend(finished);
        Ok(())
    }
}
