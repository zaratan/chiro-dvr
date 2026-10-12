use std::fs;
use std::num::NonZeroU32;
use std::ops::Range;
use std::path::Path;

use draw::style::style_for;
use draw::text::Typeface;
use files::names::Part;
use files::params::spans_in_seconds;
use reference::recording::Recording;
use serde::Deserialize;
use summary::geometry::distance;
use summary::header::{HeaderLine, duration};
use summary::layout::{MAX_TRACKS_PER_IMAGE, Mark, PageLayout, Summary, layout};
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

#[cfg(test)]
fn expected_header(lines: Vec<ExpectedLine>) -> (Vec<HeaderLine>, Option<(String, usize)>) {
    let mut header = Vec::new();
    let mut unreadable = None;
    for line in lines {
        match line {
            ExpectedLine::Name { text } => header.push(HeaderLine::Name(text)),
            ExpectedLine::Tracks { count } => header.push(HeaderLine::Tracks(count)),
            ExpectedLine::Window { start, end } => header.push(HeaderLine::Window { start, end }),
            ExpectedLine::NotAnalysed { start, end } => header.push(HeaderLine::NotAnalysed { start, end }),
            ExpectedLine::More { count } => header.push(HeaderLine::More(count)),
            ExpectedLine::Unreadable { duration, spans } => unreadable = Some((duration, spans)),
        }
    }
    (header, unreadable)
}

#[cfg(test)]
fn before_unreadable(lines: &[HeaderLine]) -> &[HeaderLine] {
    let first = lines
        .iter()
        .position(|line| matches!(line, HeaderLine::Unreadable { .. }))
        .unwrap_or(lines.len());
    &lines[..first]
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
    let periods: Vec<&[PageLayout<'_>]> = pages.chunk_by(|one, other| one.minutes == other.minutes).collect();
    if periods.len() != expected.periods.len() {
        comparison.differences.push(format!(
            "{} periods for {} in v0.1.1",
            periods.len(),
            expected.periods.len()
        ));
    }
    for (parts, period) in periods.into_iter().zip(expected.periods) {
        compare_period(&name, parts, period, &mut comparison.differences);
    }
    for page in &pages {
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
fn without_page_lines(lines: &[HeaderLine]) -> Vec<HeaderLine> {
    lines
        .iter()
        .filter(|line| {
            !matches!(
                line,
                HeaderLine::Part(_) | HeaderLine::Tracks(_) | HeaderLine::Window { .. }
            )
        })
        .cloned()
        .collect()
}

#[cfg(test)]
fn compare_period(name: &str, parts: &[PageLayout<'_>], period: ExpectedPeriod, differences: &mut Vec<String>) {
    let Some(first) = parts.first() else {
        return;
    };
    let found_suffix = suffix(first.minutes.as_ref());
    if found_suffix != period.suffix {
        differences.push(format!("{name}: suffix {found_suffix:?}, v0.1.1 {:?}", period.suffix));
    }
    let (header, unreadable) = expected_header(period.header);
    let total: f64 = first.damaged.iter().map(|[from, to]| to - from).sum();
    let found_unreadable =
        (!first.damaged.is_empty()).then(|| (duration(total.max(0.0).ceil() as u64), first.damaged.len()));
    if found_unreadable != unreadable {
        differences.push(format!(
            "{name}: unreadable {found_unreadable:?}, v0.1.1 {unreadable:?}"
        ));
    }
    let (found_lines, header) = if parts.len() > 1 {
        check_parts(name, parts, &header, differences);
        (
            without_page_lines(before_unreadable(&first.header)),
            without_page_lines(&header),
        )
    } else {
        (before_unreadable(&first.header).to_vec(), header)
    };
    if found_lines.len() != header.len() {
        differences.push(format!(
            "{name}: {} header lines, v0.1.1 {}",
            found_lines.len(),
            header.len()
        ));
    }
    for (found, expected) in found_lines.iter().zip(&header) {
        if found != expected {
            differences.push(format!("{name}: header {found:?}, v0.1.1 {expected:?}"));
        }
    }
    let marks: Vec<&Mark<'_>> = parts.iter().flat_map(|part| &part.marks).collect();
    let ids: Vec<usize> = marks.iter().map(|mark| mark.track.id()).collect();
    let expected_ids: Vec<usize> = period.tracks.iter().map(|track| track.id).collect();
    if ids != expected_ids {
        differences.push(format!("{name}: tracks {ids:?}, v0.1.1 {expected_ids:?}"));
        return;
    }
    for (mark, expected) in marks.iter().zip(&period.tracks) {
        if parts.len() == 1 && mark.color != expected.color {
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
fn check_parts(name: &str, parts: &[PageLayout<'_>], header: &[HeaderLine], differences: &mut Vec<String>) {
    let expected_tracks = header.iter().find_map(|line| match line {
        HeaderLine::Tracks(count) => Some(*count),
        _ => None,
    });
    let mut tracks = 0;
    for (index, part) in parts.iter().enumerate() {
        let numbered = part.header.contains(&HeaderLine::Part(Part {
            number: index + 1,
            count: parts.len(),
        }));
        if !numbered || part.marks.len() > MAX_TRACKS_PER_IMAGE {
            differences.push(format!(
                "{name}: part {} of {} is misnumbered or overfull",
                index + 1,
                parts.len()
            ));
        }
        tracks += part.marks.len();
    }
    if Some(tracks) != expected_tracks {
        differences.push(format!(
            "{name}: {tracks} tracks over the parts, v0.1.1 {expected_tracks:?}"
        ));
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
