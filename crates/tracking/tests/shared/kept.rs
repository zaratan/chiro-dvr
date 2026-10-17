use reference::recording::Recording;
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use types::rate::FrameRate;
use types::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Kept {
    pub(crate) fewer_detections: usize,
    pub(crate) more_detections: usize,
    pub(crate) unstable_spans_changed: usize,
}

pub(crate) const UNCHANGED: Kept = Kept {
    fewer_detections: 0,
    more_detections: 0,
    unstable_spans_changed: 0,
};

pub(crate) fn kept(recording: &Recording, rate: FrameRate) -> Kept {
    let summary = &recording.summary;
    let exclusion = exclude(
        recording.frames.detections(),
        summary.damaged_spans.clone(),
        summary.counts.damaged_frames_reported,
        rate,
        StabilityConfig::default(),
    );
    let (now, recorded) = (exclusion.detections.detection_count(), summary.counts.detections_kept);
    let changed = |spans: &[Span], others: &[Span]| spans.iter().filter(|span| !others.contains(span)).count();
    Kept {
        fewer_detections: recorded.saturating_sub(now),
        more_detections: now.saturating_sub(recorded),
        unstable_spans_changed: changed(&exclusion.unstable, &summary.unstable_spans)
            + changed(&summary.unstable_spans, &exclusion.unstable),
    }
}
