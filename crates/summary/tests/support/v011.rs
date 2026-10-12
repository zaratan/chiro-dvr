use std::fs;
use std::num::NonZeroU32;
use std::ops::Range;
use std::path::Path;

use draw::style::style_for;
use draw::text::Typeface;
use files::params::spans_in_seconds;
use reference::recording::Recording;
use serde::Deserialize;
use summary::geometry::distance;
use summary::header::HeaderLine;
use summary::layout::{PageLayout, Summary, layout};
use summary::placement::Frame;

const MARKER_GAP: f64 = 2.0;
const ROUNDING: f64 = 1e-9;

#[derive(Debug, Deserialize)]
struct Expected {
    periods: Vec<ExpectedPeriod>,
}

#[derive(Debug, Deserialize)]
struct ExpectedPeriod {
    suffix: String,
    header: Vec<ExpectedLine>,
    tracks: Vec<ExpectedTrack>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "line", rename_all = "snake_case")]
enum ExpectedLine {
    Name { text: String },
    Tracks { count: usize },
    Window { start: String, end: String },
    NotAnalysed { start: String, end: String },
    More { count: usize },
    Unreadable { duration: String, spans: usize },
}

#[derive(Debug, Deserialize)]
struct ExpectedTrack {
    id: usize,
    color: usize,
    start: String,
}

impl From<ExpectedLine> for HeaderLine {
    fn from(line: ExpectedLine) -> Self {
        match line {
            ExpectedLine::Name { text } => Self::Name(text),
            ExpectedLine::Tracks { count } => Self::Tracks(count),
            ExpectedLine::Window { start, end } => Self::Window { start, end },
            ExpectedLine::NotAnalysed { start, end } => Self::NotAnalysed { start, end },
            ExpectedLine::More { count } => Self::More(count),
            ExpectedLine::Unreadable { duration, spans } => Self::Unreadable { duration, spans },
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Comparison {
    pub(crate) tracks: usize,
    pub(crate) radius: f32,
    pub(crate) beside: usize,
    pub(crate) farthest: f64,
    pub(crate) differences: Vec<String>,
    pub(crate) misplaced: Vec<String>,
}

#[cfg(test)]
fn suffix(minutes: Option<&Range<u32>>) -> String {
    minutes.map_or_else(String::new, |range| format!("_{:03}m-{:03}m", range.start, range.end))
}

#[cfg(test)]
pub(crate) fn compare(recording: &Path, expected: &Path) -> Comparison {
    let recorded = Recording::read(recording).unwrap();
    let expected: Expected = serde_json::from_str(&fs::read_to_string(expected).unwrap()).unwrap();
    let timeline = recorded.timeline().unwrap();
    let ignored = spans_in_seconds(&recorded.summary.unstable_spans, &timeline);
    let damaged = spans_in_seconds(&recorded.summary.damaged_spans, &timeline);
    let name = Path::new(&recorded.summary.video.name)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let summary = Summary {
        name: &name,
        tracks: &recorded.tracks,
        timeline: &timeline,
        ignored: &ignored,
        damaged: &damaged,
    };
    let (width, height) = (recorded.summary.stream.width, recorded.summary.stream.height);
    let typeface = Typeface::roboto().unwrap();
    let widest = recorded
        .tracks
        .iter()
        .map(types::track::Track::id)
        .max()
        .unwrap_or_default()
        .to_string();
    let style = style_for(NonZeroU32::new(width).unwrap(), &widest, &typeface);
    let frame = Frame {
        width: f64::from(width),
        height: f64::from(height),
    };
    let pages = layout(&summary, frame, &style);
    let mut comparison = Comparison {
        tracks: recorded.tracks.len(),
        radius: style.radius,
        ..Comparison::default()
    };
    if pages.len() != expected.periods.len() {
        comparison
            .differences
            .push(format!("{} pages for {} periods", pages.len(), expected.periods.len()));
    }
    for (page, period) in pages.iter().zip(expected.periods) {
        compare_page(&name, page, period, &mut comparison.differences);
        check_markers(&name, page, frame, f64::from(style.radius), &mut comparison.misplaced);
        for mark in &page.marks {
            let away = mark
                .path
                .resampled(1.0)
                .into_iter()
                .map(|spot| distance(spot, mark.center))
                .fold(f64::INFINITY, f64::min);
            if away > f64::from(style.radius) {
                comparison.beside += 1;
            }
            comparison.farthest = comparison.farthest.max(away);
        }
    }
    comparison
}

#[cfg(test)]
fn compare_page(name: &str, page: &PageLayout<'_>, period: ExpectedPeriod, differences: &mut Vec<String>) {
    let found_suffix = suffix(page.minutes.as_ref());
    if found_suffix != period.suffix {
        differences.push(format!("{name}: suffix {found_suffix:?}, v0.1.1 {:?}", period.suffix));
    }
    let header: Vec<HeaderLine> = period.header.into_iter().map(HeaderLine::from).collect();
    if page.header.len() != header.len() {
        differences.push(format!(
            "{name}: {} header lines, v0.1.1 {}",
            page.header.len(),
            header.len()
        ));
    }
    for (found, expected) in page.header.iter().zip(&header) {
        if found != expected {
            differences.push(format!("{name}: header {found:?}, v0.1.1 {expected:?}"));
        }
    }
    let ids: Vec<usize> = page.marks.iter().map(|mark| mark.track.id()).collect();
    let expected_ids: Vec<usize> = period.tracks.iter().map(|track| track.id).collect();
    if ids != expected_ids {
        differences.push(format!("{name}: tracks {ids:?}, v0.1.1 {expected_ids:?}"));
        return;
    }
    for (mark, expected) in page.marks.iter().zip(&period.tracks) {
        if mark.color != expected.color {
            differences.push(format!(
                "{name}: track {} colour {}, v0.1.1 {}",
                expected.id, mark.color, expected.color
            ));
        }
        if mark.start != expected.start {
            differences.push(format!(
                "{name}: track {} start {}, v0.1.1 {}",
                expected.id, mark.start, expected.start
            ));
        }
    }
}

#[cfg(test)]
fn check_markers(name: &str, page: &PageLayout<'_>, frame: Frame, radius: f64, misplaced: &mut Vec<String>) {
    let inset = radius + MARKER_GAP - ROUNDING;
    for mark in &page.marks {
        let [x, y] = mark.center;
        if x < inset || y < inset || x > frame.width - inset || y > frame.height - inset {
            misplaced.push(format!(
                "{name}: marker {} at {:?} leaves the frame",
                mark.track.id(),
                mark.center
            ));
        }
    }
    for (index, mark) in page.marks.iter().enumerate() {
        for other in &page.marks[index + 1..] {
            if distance(mark.center, other.center) < 2.0 * radius - ROUNDING {
                misplaced.push(format!(
                    "{name}: markers {} at {:.0?} and {} at {:.0?} overlap ({:.1} px apart)",
                    mark.track.id(),
                    mark.center,
                    other.track.id(),
                    other.center,
                    distance(mark.center, other.center)
                ));
            }
        }
    }
}
