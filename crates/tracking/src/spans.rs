use types::detection::FrameDetections;
use types::span::Span;

use crate::error::Error;

#[cfg(test)]
mod tests;

pub(crate) fn coverage(spans: &[Span], frame_count: usize) -> Vec<bool> {
    let mut covered = vec![false; frame_count];
    for span in spans {
        let end = span.last().get().saturating_add(1).min(frame_count);
        if let Some(frames) = covered.get_mut(span.first().get()..end) {
            frames.fill(true);
        }
    }
    covered
}

pub(crate) fn without_spans(detections: &FrameDetections, covered: &[bool]) -> Result<FrameDetections, Error> {
    let kept = detections
        .iter()
        .zip(covered)
        .map(|((_, found), &covered)| if covered { Vec::new() } else { found.to_vec() })
        .collect();
    Ok(FrameDetections::new(kept)?)
}
