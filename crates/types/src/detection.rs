use crate::frame::FrameIndex;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Detection {
    pub frame: FrameIndex,
    pub x: f64,
    pub y: f64,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub area: f64,
    pub amplitude: f64,
}

impl Detection {
    pub fn distance(&self, other: &Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameDetections {
    frames: Vec<Vec<Detection>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("a detection of frame {found} is listed under frame {slot}")]
    Misplaced { slot: FrameIndex, found: FrameIndex },
}

impl FrameDetections {
    pub fn new(frames: Vec<Vec<Detection>>) -> Result<Self, Error> {
        for (slot, found) in frames.iter().enumerate() {
            let slot = FrameIndex::new(slot);
            if let Some(misplaced) = found.iter().find(|detection| detection.frame != slot) {
                return Err(Error::Misplaced {
                    slot,
                    found: misplaced.frame,
                });
            }
        }
        Ok(Self { frames })
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn detection_count(&self) -> usize {
        self.frames.iter().map(Vec::len).sum()
    }

    pub fn get(&self, frame: FrameIndex) -> Option<&[Detection]> {
        self.frames.get(frame.get()).map(Vec::as_slice)
    }

    pub fn iter(&self) -> impl Iterator<Item = (FrameIndex, &[Detection])> {
        self.frames
            .iter()
            .enumerate()
            .map(|(frame, found)| (FrameIndex::new(frame), found.as_slice()))
    }
}
