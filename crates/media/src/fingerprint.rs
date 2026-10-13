use std::hash::{DefaultHasher, Hash, Hasher};

use crate::frame::YuvFrame;

#[cfg(test)]
mod tests;

/// Comparable within one build only: `DefaultHasher` may change between Rust releases, so never store it.
pub fn of(frame: &YuvFrame) -> u64 {
    let mut hasher = DefaultHasher::new();
    (frame.width(), frame.height(), frame.range()).hash(&mut hasher);
    (frame.y(), frame.u(), frame.v()).hash(&mut hasher);
    hasher.finish()
}
