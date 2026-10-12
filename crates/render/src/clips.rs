use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::thread;

use draw::text::{self, Typeface};
use media::encode::{self, Codec};
use media::frame;
use types::track::Track;

use crate::config::RenderConfig;
use crate::encoders::{Encoders, Ffmpeg, Open};
use crate::group::{Rendering, Written};
use crate::overlay::Overlay;
use crate::picture::{self, Picture};
use crate::schedule;
use crate::source::{self, Origin, Source};
use crate::windows::{HARDWARE_LANES, Window, clip_windows, groups};

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("track {track} is not a track of the video")]
    UnknownTrack { track: usize },
    #[error("the track numbers cannot be drawn")]
    Typeface(#[source] text::Error),
    #[error("the frames of the clips cannot be read")]
    Source(#[from] source::Error),
    #[error("a frame of the clips cannot be drawn")]
    Picture(#[source] picture::Error),
    #[error("cannot write the clip {}", path.display())]
    Clip {
        path: PathBuf,
        #[source]
        source: encode::Error,
    },
    #[error("the clip {} cannot be written there, whatever the encoder", path.display())]
    Destination {
        path: PathBuf,
        #[source]
        source: encode::Error,
    },
    #[error("no encoder is left to write the clip {}", path.display())]
    NoEncoder { path: PathBuf },
    #[error("cannot hold a frame read ahead")]
    Frame(#[source] frame::Error),
    #[error("{served} frames were served instead of {expected}")]
    Unserved { served: usize, expected: usize },
    #[error("stopped because another clip failed")]
    Stopped,
}

impl Error {
    pub(crate) fn writing(path: &Path, source: encode::Error) -> Self {
        let path = path.to_path_buf();
        if source.concerns_the_destination() {
            Self::Destination { path, source }
        } else {
            Self::Clip { path, source }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clip {
    pub track: usize,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct Redone {
    pub path: PathBuf,
    pub cause: encode::Error,
}

#[derive(Debug, Default)]
pub struct Rendered {
    pub restarts: usize,
    pub redone: Vec<Redone>,
}

pub fn render(
    source: &Source,
    tracks: &[Track],
    clips: &[Clip],
    config: RenderConfig,
    codecs: &[Codec],
) -> Result<Rendered, Error> {
    let threads = thread::available_parallelism().map_or(1, |cores| cores.get() / 2);
    let video = Video { source, tracks, config };
    render_with(&Ffmpeg, threads, &video, clips, codecs)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Video<'a> {
    pub(crate) source: &'a Source,
    pub(crate) tracks: &'a [Track],
    pub(crate) config: RenderConfig,
}

pub(crate) fn render_with<O: Open>(
    open: &O,
    threads: usize,
    video: &Video<'_>,
    clips: &[Clip],
    codecs: &[Codec],
) -> Result<Rendered, Error> {
    let Video { source, tracks, config } = *video;
    let mut paths: Vec<Option<&Path>> = vec![None; tracks.len()];
    for clip in clips {
        let path = paths
            .get_mut(clip.track)
            .ok_or(Error::UnknownTrack { track: clip.track })?;
        *path = Some(&clip.path);
    }
    let (frames, rate) = (source.frame_count(), source.rate());
    let windows: Vec<Window> = clip_windows(tracks, frames, config, rate)
        .into_iter()
        .filter(|window| paths.get(window.track).copied().flatten().is_some())
        .collect();
    if let Some(clip) = clips
        .iter()
        .find(|clip| windows.iter().all(|window| window.track != clip.track))
    {
        return Err(Error::UnknownTrack { track: clip.track });
    }
    let typeface = Typeface::roboto().map_err(Error::Typeface)?;
    let overlay = Overlay::new(tracks, frames, config, rate, &typeface);
    let encoders = Encoders::new(open, codecs, rate);
    let rendering = Rendering {
        source,
        overlay: &overlay,
        encoders: &encoders,
        paths: &paths,
    };
    let written = schedule::run(
        &groups(&windows),
        threads,
        HARDWARE_LANES,
        |group| encoders.hardware_sessions(group),
        |picture: &mut Option<Picture>, group, stop| rendering.render(group, picture, stop),
    )?;
    let mut rendered = Rendered::default();
    let mut redo = Vec::new();
    for group in written {
        rendered.restarts += restarts(&group);
        redo.extend(group.redo);
    }
    let (mut picture, running) = (None, AtomicBool::new(false));
    for failed in redo {
        for group in groups(&[failed.window]) {
            let again = rendering.render(&group, &mut picture, &running)?;
            rendered.restarts += restarts(&again);
            if let Some(twice) = again.redo.into_iter().next() {
                return Err(Error::writing(&twice.path, twice.cause));
            }
        }
        rendered.redone.push(Redone {
            path: failed.path,
            cause: failed.cause,
        });
    }
    Ok(rendered)
}

fn restarts(written: &Written) -> usize {
    usize::from(matches!(written.origin, Origin::Start(_)))
}
