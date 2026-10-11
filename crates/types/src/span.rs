use crate::frame::FrameIndex;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    first: FrameIndex,
    last: FrameIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("the span ends at frame {last} before it starts at frame {first}")]
    Reversed { first: FrameIndex, last: FrameIndex },
}

impl Span {
    pub fn new(first: FrameIndex, last: FrameIndex) -> Result<Self, Error> {
        if last < first {
            return Err(Error::Reversed { first, last });
        }
        Ok(Self { first, last })
    }

    pub fn first(self) -> FrameIndex {
        self.first
    }

    pub fn last(self) -> FrameIndex {
        self.last
    }

    pub fn contains(self, frame: FrameIndex) -> bool {
        self.first <= frame && frame <= self.last
    }

    pub fn frame_count(self) -> usize {
        self.last.get() - self.first.get() + 1
    }
}
