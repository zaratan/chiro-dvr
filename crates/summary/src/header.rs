use std::fmt::Debug;

use files::names::Part;

use crate::periods::{Period, TimeRange};

#[cfg(test)]
mod tests;

pub const MAX_DATED_LINES: usize = 3;
const SECONDS_PER_MINUTE: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderLine {
    Name(String),
    Part(Part),
    Tracks(usize),
    Window { start: String, end: String },
    NotAnalysed { start: String, end: String },
    More(usize),
    Unreadable { start: String, end: String },
    MoreUnreadable { count: usize, total_seconds: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    NotAnalysed,
    Unreadable,
}

pub trait Labels: Debug {
    fn line(&self, line: &HeaderLine) -> String;
    fn band(&self, band: Band) -> String;
}

pub fn clock(seconds: f64) -> String {
    clock_whole(seconds.max(0.0).floor() as u64)
}

fn clock_whole(seconds: u64) -> String {
    format!("{}:{:02}", seconds / SECONDS_PER_MINUTE, seconds % SECONDS_PER_MINUTE)
}

pub fn duration(whole_seconds: u64) -> String {
    let (minutes, rest) = (whole_seconds / SECONDS_PER_MINUTE, whole_seconds % SECONDS_PER_MINUTE);
    if minutes == 0 {
        format!("{rest} s")
    } else {
        format!("{minutes} min {rest} s")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Scope {
    pub part: Option<Part>,
    pub tracks: usize,
    pub window: TimeRange,
}

pub fn header_lines(name: &str, period: &Period<'_>, scope: Scope) -> Vec<HeaderLine> {
    let mut lines = vec![HeaderLine::Name(name.to_owned())];
    lines.extend(scope.part.map(HeaderLine::Part));
    lines.push(HeaderLine::Tracks(scope.tracks));
    lines.push(HeaderLine::Window {
        start: clock(scope.window[0]),
        end: clock(scope.window[1]),
    });
    let (not_analysed, hidden) = dated(&period.ignored, |start, end| HeaderLine::NotAnalysed { start, end });
    lines.extend(not_analysed);
    if hidden > 0 {
        lines.push(HeaderLine::More(hidden));
    }
    let (unreadable, hidden) = dated(&period.damaged, |start, end| HeaderLine::Unreadable { start, end });
    lines.extend(unreadable);
    if hidden > 0 {
        let total: f64 = period.damaged.iter().map(|[from, to]| to - from).sum();
        lines.push(HeaderLine::MoreUnreadable {
            count: hidden,
            total_seconds: total.max(0.0).ceil() as u64,
        });
    }
    lines
}

fn dated(ranges: &[TimeRange], line: impl Fn(String, String) -> HeaderLine) -> (Vec<HeaderLine>, usize) {
    let merged = whole_seconds(ranges);
    let shown = merged
        .iter()
        .take(MAX_DATED_LINES)
        .map(|&[start, end]| line(clock_whole(start), clock_whole(end)))
        .collect();
    (shown, merged.len().saturating_sub(MAX_DATED_LINES))
}

fn whole_seconds(ranges: &[TimeRange]) -> Vec<[u64; 2]> {
    let mut rounded: Vec<[u64; 2]> = ranges
        .iter()
        .map(|[from, to]| [from.max(0.0).floor() as u64, to.max(0.0).ceil() as u64])
        .collect();
    rounded.sort_unstable();
    let mut merged: Vec<[u64; 2]> = Vec::with_capacity(rounded.len());
    for range in rounded {
        match merged.last_mut() {
            Some(last) if range[0] <= last[1] => last[1] = last[1].max(range[1]),
            _ => merged.push(range),
        }
    }
    merged
}
