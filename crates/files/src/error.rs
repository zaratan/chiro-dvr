use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot write {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{} is already there", path.display())]
    Taken { path: PathBuf },
    #[error("cannot read {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot list {}", path.display())]
    List {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot remove {}", path.display())]
    Remove {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot set aside {}", path.display())]
    SetAside {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot encode {}", path.display())]
    Encode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

impl Error {
    pub fn path(&self) -> &Path {
        match self {
            Self::Write { path, .. }
            | Self::Taken { path }
            | Self::Read { path, .. }
            | Self::List { path, .. }
            | Self::Remove { path, .. }
            | Self::SetAside { path, .. }
            | Self::Encode { path, .. } => path,
        }
    }
}
