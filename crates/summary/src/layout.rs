use std::ops::Range;

use draw::color::PALETTE;
use draw::style::SummaryStyle;
use files::names::Part;
use files::time::frame_seconds;
use types::timeline::Timeline;
use types::track::Track;

use crate::arrows::arrow_head;
use crate::colors::assign_colors;
use crate::geometry::{Spot, Trajectory};
use crate::header::{HeaderLine, Scope, clock, header_lines};
use crate::periods::{Period, TimeRange, split_by_period};
use crate::placement::{Frame, KeepClear, place_markers};

#[cfg(test)]
mod tests;

pub const MAX_TRACKS_PER_IMAGE: usize = 20;

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
    pub part: Option<Part>,
    pub header: Vec<HeaderLine>,
    pub marks: Vec<Mark<'a>>,
    pub window: TimeRange,
    pub ignored: Vec<TimeRange>,
    pub damaged: Vec<TimeRange>,
    pub other_starts: Vec<f64>,
}

struct Dated<'a> {
    track: &'a Track,
    path: Trajectory,
    start_seconds: f64,
    end_seconds: f64,
}

pub fn layout<'a>(summary: &Summary<'a>, frame: Frame, style: &SummaryStyle) -> Vec<PageLayout<'a>> {
    split_by_period(summary.tracks, summary.timeline, summary.ignored, summary.damaged)
        .into_iter()
        .flat_map(|period| period_pages(summary, &period, frame, style))
        .collect()
}

fn period_pages<'a>(
    summary: &Summary<'a>,
    period: &Period<'a>,
    frame: Frame,
    style: &SummaryStyle,
) -> Vec<PageLayout<'a>> {
    let dated: Vec<Dated<'a>> = period
        .tracks
        .iter()
        .map(|&track| Dated {
            track,
            path: Trajectory::of(track),
            start_seconds: frame_seconds(summary.timeline, track.first().frame),
            end_seconds: frame_seconds(summary.timeline, track.last().frame),
        })
        .collect();
    let packs: Vec<&[Dated<'a>]> = if dated.is_empty() {
        vec![&[]]
    } else {
        let parts = dated.len().div_ceil(MAX_TRACKS_PER_IMAGE);
        dated.chunks(dated.len().div_ceil(parts)).collect()
    };
    let count = packs.len();
    packs
        .iter()
        .enumerate()
        .map(|(index, pack)| {
            let part = (count > 1).then_some(Part {
                number: index + 1,
                count,
            });
            let window = match (part, pack.first()) {
                (Some(_), Some(first)) => [
                    first.start_seconds,
                    pack.iter()
                        .map(|one| one.end_seconds)
                        .fold(first.start_seconds, f64::max)
                        .ceil(),
                ],
                _ => [period.start, period.end],
            };
            let scope = Scope {
                part,
                tracks: pack.len(),
                window,
            };
            let other_starts = packs
                .iter()
                .enumerate()
                .filter(|&(other, _)| other != index)
                .flat_map(|(_, other)| other.iter().map(|one| one.start_seconds))
                .collect();
            PageLayout {
                minutes: period.minutes(),
                part,
                header: header_lines(summary.name, period, scope),
                marks: marks(pack, frame, style),
                window: [period.start, period.end],
                ignored: period.ignored.clone(),
                damaged: period.damaged.clone(),
                other_starts,
            }
        })
        .collect()
}

fn marks<'a>(pack: &[Dated<'a>], frame: Frame, style: &SummaryStyle) -> Vec<Mark<'a>> {
    let radius = f64::from(style.radius);
    let arrow_length = f64::from(style.arrow_length);
    let paths: Vec<Trajectory> = pack.iter().map(|one| one.path.clone()).collect();
    let heads: Vec<Option<[Spot; 3]>> = paths.iter().map(|path| arrow_head(path, arrow_length)).collect();
    let tips: Vec<Spot> = heads.iter().flatten().map(|head| head[0]).collect();
    let keep_clear = KeepClear {
        spots: &tips,
        radius: radius + arrow_length,
    };
    let centers = place_markers(&paths, frame, radius, keep_clear);
    let colors = assign_colors(&paths, PALETTE.len(), f64::from(style.neighbor_distance));
    pack.iter()
        .zip(paths)
        .zip(heads)
        .zip(centers)
        .zip(colors)
        .map(|((((one, path), head), center), color)| Mark {
            track: one.track,
            anchor: path.nearest(center),
            path,
            color,
            start: clock(one.start_seconds),
            head,
            center,
            start_seconds: one.start_seconds,
        })
        .collect()
}
