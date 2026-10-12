use std::ops::Range;

use draw::color::PALETTE;
use draw::style::SummaryStyle;
use files::time::frame_seconds;
use types::timeline::Timeline;
use types::track::Track;

use crate::arrows::arrow_head;
use crate::colors::assign_colors;
use crate::geometry::{Spot, Trajectory};
use crate::header::{HeaderLine, clock, header_lines};
use crate::periods::{TimeRange, split_by_period};
use crate::placement::{Frame, KeepClear, place_markers};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy)]
pub struct Summary<'a> {
    pub name: &'a str,
    pub tracks: &'a [Track],
    pub timeline: &'a Timeline,
    pub ignored: &'a [TimeRange],
    pub damaged: &'a [TimeRange],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mark<'a> {
    pub track: &'a Track,
    pub path: Trajectory,
    pub color: usize,
    pub start: String,
    pub head: Option<[Spot; 3]>,
    pub center: Spot,
    pub anchor: Spot,
    pub start_seconds: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageLayout<'a> {
    pub minutes: Option<Range<u32>>,
    pub header: Vec<HeaderLine>,
    pub marks: Vec<Mark<'a>>,
    pub window: TimeRange,
    pub ignored: Vec<TimeRange>,
    pub damaged: Vec<TimeRange>,
}

pub fn layout<'a>(summary: &Summary<'a>, frame: Frame, style: &SummaryStyle) -> Vec<PageLayout<'a>> {
    let radius = f64::from(style.radius);
    let arrow_length = f64::from(style.arrow_length);
    split_by_period(summary.tracks, summary.timeline, summary.ignored, summary.damaged)
        .into_iter()
        .map(|period| {
            let paths: Vec<Trajectory> = period.tracks.iter().map(|track| Trajectory::of(track)).collect();
            let colors = assign_colors(&paths, PALETTE.len(), f64::from(style.neighbor_distance));
            let heads: Vec<Option<[Spot; 3]>> = paths.iter().map(|path| arrow_head(path, arrow_length)).collect();
            let tips: Vec<Spot> = heads.iter().flatten().map(|head| head[0]).collect();
            let keep_clear = KeepClear {
                spots: &tips,
                radius: radius + arrow_length,
            };
            let centers = place_markers(&paths, frame, radius, keep_clear);
            let marks = period
                .tracks
                .iter()
                .zip(paths)
                .zip(colors)
                .zip(heads)
                .zip(centers)
                .map(|((((&track, path), color), head), center)| {
                    let start_seconds = frame_seconds(summary.timeline, track.first().frame);
                    Mark {
                        track,
                        anchor: path.nearest(center),
                        path,
                        color,
                        start: clock(start_seconds),
                        head,
                        center,
                        start_seconds,
                    }
                })
                .collect();
            PageLayout {
                minutes: period.minutes(),
                header: header_lines(summary.name, &period),
                marks,
                window: [period.start, period.end],
                ignored: period.ignored,
                damaged: period.damaged,
            }
        })
        .collect()
}
