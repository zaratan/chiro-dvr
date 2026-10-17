use std::fs;
use std::path::Path;

const BEYOND_ANY_DISK: u64 = 10_000_000_000_000;

#[cfg(test)]
pub(crate) fn stretched_beyond_any_disk(video: &Path) {
    fs::OpenOptions::new()
        .write(true)
        .open(video)
        .unwrap()
        .set_len(BEYOND_ANY_DISK)
        .unwrap();
}
