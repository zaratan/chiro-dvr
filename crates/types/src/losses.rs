use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Frames the container announces but no decoder gives, before the first decoded frame and after the last.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeLosses {
    pub before_first: usize,
    pub after_last: usize,
}

impl EdgeLosses {
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self {
            before_first: self.before_first.max(other.before_first),
            after_last: self.after_last.max(other.after_last),
        }
    }
}
