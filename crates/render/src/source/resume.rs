use types::frame::{FrameIndex, Pts};

use super::numbering::Numbering;

#[cfg(test)]
mod tests;

pub(super) fn clean_key_before(
    first: FrameIndex,
    keys: &[FrameIndex],
    errors: &[FrameIndex],
    numbering: &Numbering,
) -> Option<(FrameIndex, Pts)> {
    let reached = keys.partition_point(|key| *key <= first);
    keys[..reached]
        .iter()
        .rev()
        .filter(|key| errors.binary_search(key).is_err())
        .find_map(|key| numbering.pts(*key).map(|pts| (*key, pts)))
}
