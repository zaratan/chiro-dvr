use types::frame::{FrameIndex, Pts};
use types::timeline::Timeline;

use super::Error;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct Numbering {
    known: Vec<(Pts, FrameIndex)>,
}

impl Numbering {
    pub(super) fn of(timeline: &Timeline) -> Result<Self, Error> {
        let known: Vec<(Pts, FrameIndex)> = (0..timeline.frame_count())
            .map(FrameIndex::new)
            .filter_map(|frame| timeline.pts(frame).map(|pts| (pts, frame)))
            .collect();
        if let Some(pair) = known.windows(2).find(|pair| pair[1].0 <= pair[0].0) {
            return Err(Error::UnorderedPts { frame: pair[1].1 });
        }
        Ok(Self { known })
    }

    pub(super) fn frame(&self, pts: Pts) -> Option<FrameIndex> {
        let found = self.known.binary_search_by_key(&pts, |(known, _)| *known).ok()?;
        Some(self.known[found].1)
    }

    pub(super) fn pts(&self, frame: FrameIndex) -> Option<Pts> {
        let found = self.known.binary_search_by_key(&frame, |(_, known)| *known).ok()?;
        Some(self.known[found].0)
    }
}
