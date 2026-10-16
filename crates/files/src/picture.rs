use std::fs::File;
use std::io::Read;
use std::path::Path;

#[cfg(test)]
mod tests;

const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const HEADER_CHUNK: &[u8; 4] = b"IHDR";
const HEADER_BYTES: usize = 24;

pub fn png_size(path: &Path) -> Option<(u32, u32)> {
    let mut head = [0_u8; HEADER_BYTES];
    File::open(path).ok()?.read_exact(&mut head).ok()?;
    if &head[..8] != SIGNATURE || &head[12..16] != HEADER_CHUNK {
        return None;
    }
    let width = u32::from_be_bytes(head[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(head[20..24].try_into().ok()?);
    Some((width, height))
}
