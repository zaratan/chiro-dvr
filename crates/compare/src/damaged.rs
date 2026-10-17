use engine::damage::{Damage, Reports};
use reference::frames::FrameEntry;
use reference::video::VideoSummary;
use types::frame::FrameIndex;

#[cfg(test)]
mod tests;

pub fn reported_by(summary: &VideoSummary) -> Reports {
    match (&summary.decoder, &summary.ffprobe) {
        (Some(decoder), _) => Reports {
            error_frames: decoder.error_frames.clone(),
            jump_frames: decoder.gap_frames.clone(),
            edges: decoder.losses,
        },
        (None, Some(probe)) => Reports {
            error_frames: probe.error_frames.clone(),
            jump_frames: probe.gap_frames.clone(),
            ..Reports::default()
        },
        (None, None) => Reports::default(),
    }
}

/// The damage both readings report together, its spans drawn again by the current rule.
pub fn joined(reference: &Reports, current: &Reports, entries: &[FrameEntry], margin: usize) -> Damage {
    let both = |reference: &[FrameIndex], current: &[FrameIndex]| reference.iter().chain(current).copied().collect();
    let keys: Vec<FrameIndex> = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.key_frame)
        .map(|(frame, _)| FrameIndex::new(frame))
        .collect();
    Damage::reported(
        Reports {
            error_frames: both(&reference.error_frames, &current.error_frames),
            jump_frames: both(&reference.jump_frames, &current.jump_frames),
            edges: reference.edges.union(current.edges),
        },
        &keys,
        entries.len(),
        margin,
    )
}
