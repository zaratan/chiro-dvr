use engine::damage::Reports;
use engine::detection::Detected;
use reference::error::Error;
use reference::frames::{FrameEntry, Frames};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Pass {
    pub frames: Frames,
    pub reports: Reports,
}

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

pub fn pass_of(detected: &Detected) -> Result<Pass, Error> {
    Ok(Pass {
        frames: frames_of(detected)?,
        reports: detected.damage().reports().clone(),
    })
}
