use std::io;
use std::path::PathBuf;

use engine::{detection, settings};

use crate::inputs::InputGap;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reference(#[from] reference::error::Error),
    #[error(transparent)]
    Fixture(#[from] reference::fixture::Error),
    #[error(transparent)]
    Detection(#[from] detection::Error),
    #[error("{video}: the recorded detection settings cannot configure the engine")]
    Settings {
        video: String,
        #[source]
        source: settings::Error,
    },
    #[error("{video}: the reference has no valid frame rate")]
    Rate {
        video: String,
        #[source]
        source: types::rate::Error,
    },
    #[error("{video}: the reference has no valid frame size")]
    Size { video: String },
    #[error("cannot track the detections of {video}")]
    Tracking {
        video: String,
        #[source]
        source: tracking::error::Error,
    },
    #[error("{video}: the background half window is {found} frames, the reference used {expected}")]
    HalfWindow {
        video: String,
        expected: usize,
        found: usize,
    },
    #[error("{video}: the frames read now are not those of the reference, {gap}")]
    Inputs { video: String, gap: InputGap },
    #[error("{video}: media reads a stream of {found}, the v0.1.1 recorded {expected}")]
    Stream {
        video: String,
        expected: String,
        found: String,
    },
    #[error("{} would be written over {}, which holds a reference or the earlier pass", out.display(), protected.display())]
    Overlap { out: PathBuf, protected: PathBuf },
    #[error("{video}: cannot write the settings and versions of the recording")]
    Section {
        video: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("cannot create {}", path.display())]
    Folder {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
