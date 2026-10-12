use std::fs;
use std::io;
use std::ops::Range;
use std::path::Path;

#[cfg(test)]
mod tests;

const MEDIA_DATA: &[u8] = b"mdat";
const HEADER: usize = 8;
const WIPED: u8 = 0xff;

/// Overwrites parts `parts` of `count` equal parts of an mp4 payload: the frames there are lost or concealed.
pub fn wipe(path: &Path, parts: Range<usize>, count: usize) -> io::Result<()> {
    let invalid = |reason: &str| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {reason}", path.display()));
    let mut bytes = fs::read(path)?;
    let tag = bytes
        .windows(MEDIA_DATA.len())
        .position(|window| window == MEDIA_DATA)
        .ok_or_else(|| invalid("no mdat box"))?;
    let box_start = tag
        .checked_sub(HEADER - MEDIA_DATA.len())
        .ok_or_else(|| invalid("truncated mdat box"))?;
    let size: [u8; 4] = bytes
        .get(box_start..tag)
        .and_then(|size| size.try_into().ok())
        .ok_or_else(|| invalid("truncated mdat box"))?;
    let share = usize::try_from(u32::from_be_bytes(size))
        .ok()
        .and_then(|size| size.checked_sub(HEADER))
        .and_then(|payload| payload.checked_div(count))
        .ok_or_else(|| invalid("unusable mdat size"))?;
    let payload = tag + MEDIA_DATA.len();
    let offset = |part: usize| share.checked_mul(part).and_then(|bytes| bytes.checked_add(payload));
    let wiped = offset(parts.start)
        .zip(offset(parts.end))
        .filter(|(start, end)| start < end)
        .and_then(|(start, end)| bytes.get_mut(start..end))
        .ok_or_else(|| invalid("no part of the payload to wipe"))?;
    wiped.fill(WIPED);
    fs::write(path, bytes)
}
