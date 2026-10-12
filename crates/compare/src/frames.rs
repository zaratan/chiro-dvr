use engine::detection::Detected;
use reference::error::Error;
use reference::frames::{FrameEntry, Frames};

#[cfg(test)]
mod tests;

pub fn frames_of(detected: &Detected) -> Result<Frames, Error> {
    let entries = detected
        .reading()
        .frames()
        .iter()
        .map(|info| FrameEntry {
            pts: info.pts(),
            key_frame: info.is_key(),
            decode_error: !info.decode_errors().is_empty(),
        })
        .collect();
    Frames::new(entries, detected.detections().clone())
}
