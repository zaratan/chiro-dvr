use std::ops::Range;

use files::time::frame_seconds;
use types::frame::FrameIndex;
use types::timeline::Timeline;
use types::track::Track;

#[cfg(test)]
mod tests;

pub const SPAN_SECONDS: f64 = 600.0;
const SECONDS_PER_MINUTE: f64 = 60.0;

pub type TimeRange = [f64; 2];

#[derive(Debug, Clone, PartialEq)]
pub struct Period<'a> {
    pub start: f64,
    pub end: f64,
    pub tracks: Vec<&'a Track>,
    pub alone: bool,
    pub ignored: Vec<TimeRange>,
    pub damaged: Vec<TimeRange>,
}

impl Period<'_> {
    pub fn minutes(&self) -> Option<Range<u32>> {
        let first = (self.start / SECONDS_PER_MINUTE).floor() as u32;
        let last = (self.end / SECONDS_PER_MINUTE).ceil() as u32;
        (!self.alone).then_some(first..last)
    }
}

pub fn clipped(ranges: &[TimeRange], start: f64, end: f64) -> Vec<TimeRange> {
    ranges
        .iter()
        .filter(|[from, to]| *from < end && *to > start)
        .map(|[from, to]| [from.max(start), to.min(end)])
        .collect()
}

pub fn video_seconds(timeline: &Timeline) -> f64 {
    frame_seconds(timeline, FrameIndex::new(timeline.frame_count()))
}

pub fn split_by_period<'a>(
    tracks: &'a [Track],
    timeline: &Timeline,
    ignored: &[TimeRange],
    damaged: &[TimeRange],
) -> Vec<Period<'a>> {
    let duration = video_seconds(timeline);
    let count = (duration / SPAN_SECONDS).ceil().max(1.0) as usize;
    let mut groups: Vec<Vec<&Track>> = vec![Vec::new(); count];
    for track in tracks {
        let start = frame_seconds(timeline, track.first().frame);
        let group = ((start / SPAN_SECONDS).floor().max(0.0) as usize).min(count - 1);
        groups[group].push(track);
    }
    groups
        .into_iter()
        .enumerate()
        .map(|(index, tracks)| {
            let start = index as f64 * SPAN_SECONDS;
            let end = (start + SPAN_SECONDS).min(duration);
            Period {
                start,
                end,
                tracks,
                alone: count == 1,
                ignored: clipped(ignored, start, end),
                damaged: clipped(damaged, start, end),
            }
        })
        .collect()
}
