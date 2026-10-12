use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reference(#[from] reference::error::Error),
    #[error(transparent)]
    Fixture(#[from] reference::fixture::Error),
    #[error(transparent)]
    Detection(#[from] engine::detection::Error),
    #[error("{video}: the reference has no valid frame rate")]
    Rate {
        video: String,
        #[source]
        source: types::rate::Error,
    },
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
    #[error("cannot create {}", path.display())]
    Folder {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
