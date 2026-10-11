use std::fmt;

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FrameIndex(usize);

impl FrameIndex {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn get(self) -> usize {
        self.0
    }

    pub const fn abs_diff(self, other: Self) -> usize {
        self.0.abs_diff(other.0)
    }
}

impl fmt::Display for FrameIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pts(i64);

impl Pts {
    pub const fn new(ticks: i64) -> Self {
        Self(ticks)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}
