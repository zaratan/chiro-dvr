use crate::frame::{ColorRange, YuvFrame};

#[cfg(test)]
mod tests;

const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;
const WORD: usize = 8;

/// FNV-1a over little-endian 64-bit words, each step folded down: the same on every system and Rust release, so a test may store it.
pub fn of(frame: &YuvFrame) -> u64 {
    let range: u8 = match frame.range() {
        ColorRange::Limited => 0,
        ColorRange::Full => 1,
        ColorRange::Unspecified => 2,
    };
    let start = mixed(
        mixed(OFFSET, &frame.width().to_le_bytes()),
        &frame.height().to_le_bytes(),
    );
    [frame.y(), frame.u(), frame.v()]
        .into_iter()
        .fold(mixed(start, &[range]), mixed)
}

pub fn of_pixels(width: u32, height: u32, pixels: &[u8]) -> u64 {
    mixed(
        mixed(mixed(OFFSET, &width.to_le_bytes()), &height.to_le_bytes()),
        pixels,
    )
}

pub fn of_sequence(fingerprints: impl IntoIterator<Item = u64>) -> u64 {
    fingerprints.into_iter().fold(OFFSET, step)
}

fn mixed(hash: u64, bytes: &[u8]) -> u64 {
    let (words, rest) = bytes.as_chunks::<WORD>();
    let hash = words
        .iter()
        .fold(hash, |hash, word| step(hash, u64::from_le_bytes(*word)));
    let hash = rest.iter().fold(hash, |hash, byte| step(hash, u64::from(*byte)));
    step(hash, u64::try_from(bytes.len()).unwrap_or(u64::MAX))
}

fn step(hash: u64, word: u64) -> u64 {
    let multiplied = (hash ^ word).wrapping_mul(PRIME);
    multiplied ^ (multiplied >> 32)
}
