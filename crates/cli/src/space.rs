use std::io;
use std::path::Path;

#[cfg(test)]
mod tests;

#[cfg(unix)]
pub fn free_bytes(folder: &Path) -> io::Result<u64> {
    let stats = rustix::fs::statvfs(folder)?;
    Ok(stats.f_bavail.saturating_mul(stats.f_frsize))
}

#[cfg(not(unix))]
pub fn free_bytes(_folder: &Path) -> io::Result<u64> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}
