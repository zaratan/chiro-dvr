use std::path::{Path, PathBuf};
use std::thread;

use draw::text::{self, Typeface};
use media::encode::{self, Codec, StampsError};
use media::frame;
use types::rational::DivisionError;
use types::track::Track;

use crate::config::RenderConfig;
use crate::encoders::{Encoders, Ffmpeg, Open};
use crate::group::{Rendering, Scratch};
use crate::overlay::Overlay;
use crate::picture;
use crate::schedule::{self, Alone};
use crate::source::{self, Source};
use crate::windows::{Window, clip_windows, groups};
use crate::work::{Paths, Work, works};

#[cfg(test)]
mod tests;

const CORES_PER_THREAD: usize = 2;
const HARDWARE_SESSIONS: usize = 8;

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
    #[error("a {found_width}×{found_height} frame reached a zoom made for {width}×{height}")]
    ZoomSize {
        width: u32,
        height: u32,
        found_width: u32,
        found_height: u32,
    },
    #[error("the frames of {} cannot keep the source timestamps", path.display())]
    Stamps {
        path: PathBuf,
        #[source]
        source: StampsError,
    },
    #[error("the zoom cannot be slowed down")]
    SlowMotion {
        #[source]
        source: DivisionError,
    },
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outputs {
    pub clips: Vec<Clip>,
    pub zooms: Vec<Clip>,
    pub annotated: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Redone {
    pub paths: Vec<PathBuf>,
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
    outputs: &Outputs,
    config: RenderConfig,
    codecs: &[Codec],
) -> Result<Rendered, Error> {
    let threads = thread::available_parallelism().map_or(1, |cores| cores.get() / CORES_PER_THREAD);
    let video = Video { source, tracks, config };
    render_with(&Ffmpeg, threads, &video, outputs, codecs)
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
    outputs: &Outputs,
    codecs: &[Codec],
) -> Result<Rendered, Error> {
    let Video { source, tracks, config } = *video;
    let paths = by_track(&outputs.clips, tracks.len())?;
    let zooms = by_track(&outputs.zooms, tracks.len())?;
    let (frames, rate) = (source.frame_count(), source.rate());
    let windows = clip_windows(tracks, frames, config, rate);
    let wanted = |paths: &[Option<&Path>]| -> Vec<Window> {
        windows
            .iter()
            .copied()
            .filter(|window| paths.get(window.track).copied().flatten().is_some())
            .collect()
    };
    let (clip_windows, zoom_windows) = (wanted(&paths), wanted(&zooms));
    if let Some(clip) = outputs
        .clips
        .iter()
        .chain(&outputs.zooms)
        .find(|clip| windows.iter().all(|window| window.track != clip.track))
    {
        return Err(Error::UnknownTrack { track: clip.track });
    }
    let typeface = Typeface::roboto().map_err(Error::Typeface)?;
    let overlay = Overlay::new(tracks, frames, config, rate, &typeface);
    let encoders = Encoders::new(open, codecs, rate);
    let table = Paths {
        clips: &paths,
        zooms: &zooms,
    };
    let rendering = Rendering {
        source,
        overlay: &overlay,
        encoders: &encoders,
        paths: table,
        tracks,
        trail_frames: config.trail_frames(rate),
    };
    let works = works(
        groups(&clip_windows),
        &zoom_windows,
        outputs.annotated.as_deref().filter(|_| frames > 0),
    );
    let written = schedule::run(
        &works,
        threads,
        HARDWARE_SESSIONS,
        |work| match work {
            Work::Annotated(_) | Work::Clips(_) => encoders.hardware_sessions(),
            Work::Zoom(_) => 0,
        },
        |scratch: &mut Scratch, work, lease| rendering.render(work, scratch, lease),
    )?;
    let mut rendered = Rendered::default();
    let mut scratch = Scratch::default();
    for (work, written) in works.iter().zip(written) {
        rendered.restarts += written.restarts;
        let Some(cause) = written.failed else {
            continue;
        };
        let again = rendering.render(work, &mut scratch, &Alone)?;
        let paths = work.paths(table);
        if let Some(twice) = again.failed {
            let lead = paths.into_iter().next().unwrap_or_default();
            return Err(Error::writing(&lead, twice));
        }
        rendered.restarts += again.restarts;
        rendered.redone.push(Redone { paths, cause });
    }
    Ok(rendered)
}

fn by_track(clips: &[Clip], tracks: usize) -> Result<Vec<Option<&Path>>, Error> {
    let mut paths: Vec<Option<&Path>> = vec![None; tracks];
    for clip in clips {
        let path = paths
            .get_mut(clip.track)
            .ok_or(Error::UnknownTrack { track: clip.track })?;
        *path = Some(&clip.path);
    }
    Ok(paths)
}
