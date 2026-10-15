use std::io;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Refused,
    NotAFolder,
    Missing,
    ReadOnly,
    DiskFull,
    Unavailable,
}

impl From<io::ErrorKind> for Access {
    fn from(kind: io::ErrorKind) -> Self {
        match kind {
            io::ErrorKind::PermissionDenied => Self::Refused,
            io::ErrorKind::NotADirectory => Self::NotAFolder,
            io::ErrorKind::NotFound => Self::Missing,
            io::ErrorKind::ReadOnlyFilesystem => Self::ReadOnly,
            io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded => Self::DiskFull,
            _ => Self::Unavailable,
        }
    }
}
