use std::path::Path;

use crate::space::Volume;

#[cfg(test)]
mod tests;

#[cfg(unix)]
pub fn volume(folder: &Path) -> Option<Volume> {
    use std::os::unix::fs::MetadataExt;

    let existing = crate::writable::nearest_existing(folder)?;
    let device = std::fs::metadata(existing).ok()?.dev();
    Some(Volume {
        device,
        free: free_bytes(existing)?,
    })
}

#[cfg(target_os = "macos")]
fn free_bytes(existing: &Path) -> Option<u64> {
    let stats = rustix::fs::statfs(existing).ok()?;
    Some(stats.f_bavail.saturating_mul(u64::from(stats.f_bsize)))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn free_bytes(existing: &Path) -> Option<u64> {
    let stats = rustix::fs::statvfs(existing).ok()?;
    Some(stats.f_bavail.saturating_mul(stats.f_frsize))
}

#[cfg(not(unix))]
pub const fn volume(_folder: &Path) -> Option<Volume> {
    None
}
