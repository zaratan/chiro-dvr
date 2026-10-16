use types::frame::FrameIndex;
use types::span::Span;
use types::timeline::Timeline;
use types::track::Track;

#[cfg(test)]
mod tests;

const CENTISECONDS_PER_SECOND: u64 = 100;
const CENTISECONDS_PER_MINUTE: u64 = 6000;
const HUNDREDTHS: f64 = 100.0;

pub fn clock(seconds: f64) -> String {
    let centiseconds = (seconds.max(0.0) * HUNDREDTHS).round() as u64;
    let minutes = centiseconds / CENTISECONDS_PER_MINUTE;
    let rest = centiseconds % CENTISECONDS_PER_MINUTE;
    format!(
        "{minutes}:{:02}.{:02}",
        rest / CENTISECONDS_PER_SECOND,
        rest % CENTISECONDS_PER_SECOND
    )
}

pub(crate) fn hundredths(seconds: f64) -> f64 {
    (seconds * HUNDREDTHS).round() / HUNDREDTHS
}

pub fn frame_seconds(timeline: &Timeline, frame: FrameIndex) -> f64 {
    let latest = frame.get().min(timeline.frame_count().saturating_sub(1));
    let known = (0..=latest).rev().find_map(|index| {
        let index = FrameIndex::new(index);
        timeline.seconds_by_pts(index).map(|seconds| (index, seconds))
    });
    match known {
        Some((index, seconds)) => seconds + index.abs_diff(frame) as f64 * frame_duration(timeline),
        None => timeline.seconds_by_rate(frame),
    }
}

pub fn track_seconds(track: &Track, timeline: &Timeline) -> [f64; 2] {
    let start = frame_seconds(timeline, track.first().frame);
    let end = frame_seconds(timeline, track.last().frame);
    [start, (end - start).max(frame_duration(timeline))]
}

pub(crate) fn frame_duration(timeline: &Timeline) -> f64 {
    timeline.rate().seconds(FrameIndex::new(1))
}

pub(crate) fn span_seconds(span: Span, timeline: &Timeline) -> [f64; 2] {
    let after = FrameIndex::new(span.last().get().saturating_add(1));
    [frame_seconds(timeline, span.first()), frame_seconds(timeline, after)]
}
