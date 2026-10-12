use std::fmt::Debug;

use crate::periods::Period;

#[cfg(test)]
mod tests;

pub const MAX_IGNORED_LINES: usize = 3;
const SECONDS_PER_MINUTE: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderLine {
    Name(String),
    Tracks(usize),
    Window { start: String, end: String },
    NotAnalysed { start: String, end: String },
    More(usize),
    Unreadable { duration: String, spans: usize },
}

pub trait Labels: Debug {
    fn line(&self, line: &HeaderLine) -> String;
}

pub fn clock(seconds: f64) -> String {
    let whole = seconds.max(0.0).floor() as u64;
    format!("{}:{:02}", whole / SECONDS_PER_MINUTE, whole % SECONDS_PER_MINUTE)
}

pub fn duration(seconds: f64) -> String {
    let whole = seconds.max(0.0).ceil() as u64;
    let (minutes, rest) = (whole / SECONDS_PER_MINUTE, whole % SECONDS_PER_MINUTE);
    if minutes == 0 {
        format!("{rest} s")
    } else {
        format!("{minutes} min {rest} s")
    }
}

pub fn header_lines(name: &str, period: &Period<'_>) -> Vec<HeaderLine> {
    let mut lines = vec![
        HeaderLine::Name(name.to_owned()),
        HeaderLine::Tracks(period.tracks.len()),
        HeaderLine::Window {
            start: clock(period.start),
            end: clock(period.end),
        },
    ];
    lines.extend(
        period
            .ignored
            .iter()
            .take(MAX_IGNORED_LINES)
            .map(|[from, to]| HeaderLine::NotAnalysed {
                start: clock(*from),
                end: clock(to.ceil()),
            }),
    );
    if let Some(hidden) = period
        .ignored
        .len()
        .checked_sub(MAX_IGNORED_LINES)
        .filter(|&hidden| hidden > 0)
    {
        lines.push(HeaderLine::More(hidden));
    }
    if !period.damaged.is_empty() {
        lines.push(HeaderLine::Unreadable {
            duration: duration(period.damaged.iter().map(|[from, to]| to - from).sum()),
            spans: period.damaged.len(),
        });
    }
    lines
}
