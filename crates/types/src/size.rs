use std::num::NonZeroU32;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSize {
    width: NonZeroU32,
    height: NonZeroU32,
}

impl SourceSize {
    pub const fn new(width: NonZeroU32, height: NonZeroU32) -> Self {
        Self { width, height }
    }

    pub const fn width(self) -> NonZeroU32 {
        self.width
    }

    pub const fn height(self) -> NonZeroU32 {
        self.height
    }
}
