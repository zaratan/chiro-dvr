use std::fmt::Debug;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;

use tempfile::TempPath;
use types::language::Language;
use types::timeline::Timeline;
use types::track::Track;

use crate::dialect::Dialect;
use crate::error::Error;
use crate::names::partial_path;
use crate::time::{clock, frame_duration, frame_seconds};

#[cfg(test)]
mod tests;

const LEGACY_HEADER: &str =
    "id,start,end,start_s,duration_s,hits,chord_px,path_px,speed_px_s,max_area_px,peak_amplitude,filled_frames";
const LEGACY_SEPARATOR: char = ',';
const LINE_END: &str = "\r\n";
const LANGUAGES: [Language; 2] = [Language::French, Language::English];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Track,
    Start,
    End,
    Duration,
    Verdict,
    Hits,
    Chord,
    Path,
    Speed,
    MaxArea,
    Amplitude,
    FilledFrames,
    StartSeconds,
}

pub const COLUMNS: [Column; 13] = [
    Column::Track,
    Column::Start,
    Column::End,
    Column::Duration,
    Column::Verdict,
    Column::Hits,
    Column::Chord,
    Column::Path,
    Column::Speed,
    Column::MaxArea,
    Column::Amplitude,
    Column::FilledFrames,
    Column::StartSeconds,
];

pub trait Headings: Debug {
    fn heading(&self, column: Column, language: Language) -> String;
}

pub fn write_tracks_csv(
    path: &Path,
    tracks: &[Track],
    timeline: &Timeline,
    language: Language,
    headings: &dyn Headings,
) -> Result<(), Error> {
    let failed = |source| Error::Write {
        path: path.to_path_buf(),
        source,
    };
    let dialect = Dialect::of(language);
    let partial = TempPath::try_from_path(partial_path(path)).map_err(failed)?;
    let mut file = BufWriter::new(File::create(&partial).map_err(failed)?);
    write!(file, "{}{LINE_END}", header(language, headings)).map_err(failed)?;
    for track in tracks {
        write!(file, "{}{LINE_END}", row(track, timeline, dialect)).map_err(failed)?;
    }
    file.flush().map_err(failed)?;
    drop(file);
    partial
        .persist_noclobber(path)
        .map_err(|failure| match failure.error.kind() {
            io::ErrorKind::AlreadyExists => Error::Taken {
                path: path.to_path_buf(),
            },
            _ => failed(failure.error),
        })
}

pub fn is_tracks_table(path: &Path, headings: &dyn Headings) -> Result<bool, Error> {
    let bytes = fs::read(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let Ok(text) = str::from_utf8(&bytes) else {
        return Ok(false);
    };
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return Ok(false);
    };
    let legacy = Form::legacy();
    let ours = LANGUAGES.map(|language| Form::ours(language, headings));
    let form = ours.iter().chain([&legacy]).find(|form| form.header == first);
    Ok(form.is_some_and(|form| lines.all(|line| form.fits(line))))
}

#[derive(Debug)]
struct Form {
    header: String,
    separator: char,
    columns: usize,
    verdict: Option<usize>,
}

impl Form {
    fn ours(language: Language, headings: &dyn Headings) -> Self {
        Self {
            header: header(language, headings),
            separator: Dialect::of(language).separator(),
            columns: COLUMNS.len(),
            verdict: COLUMNS.iter().position(|&column| column == Column::Verdict),
        }
    }

    fn legacy() -> Self {
        Self {
            header: LEGACY_HEADER.to_owned(),
            separator: LEGACY_SEPARATOR,
            columns: LEGACY_HEADER.split(LEGACY_SEPARATOR).count(),
            verdict: None,
        }
    }

    fn fits(&self, line: &str) -> bool {
        let cells: Vec<&str> = line.split(self.separator).collect();
        cells.len() == self.columns
            && self
                .verdict
                .is_none_or(|index| cells.get(index).is_some_and(|cell| cell.is_empty()))
    }
}

fn header(language: Language, headings: &dyn Headings) -> String {
    Dialect::of(language).header(COLUMNS.map(|column| headings.heading(column, language)))
}

fn row(track: &Track, timeline: &Timeline, dialect: Dialect) -> String {
    let measured = Measured::of(track, timeline);
    dialect.row(COLUMNS.map(|column| measured.cell(column, dialect)))
}

struct Measured<'a> {
    track: &'a Track,
    start: f64,
    end: f64,
    duration: f64,
    path: f64,
    filled: usize,
}

impl<'a> Measured<'a> {
    fn of(track: &'a Track, timeline: &Timeline) -> Self {
        let (first, last) = (track.first(), track.last());
        let start = frame_seconds(timeline, first.frame);
        let end = frame_seconds(timeline, last.frame);
        let spanned = last.frame.get() - first.frame.get() + 1;
        Self {
            track,
            start,
            end,
            duration: (end - start).max(frame_duration(timeline)),
            path: track.path_length(),
            filled: spanned - track.points().len(),
        }
    }

    fn cell(&self, column: Column, dialect: Dialect) -> String {
        match column {
            Column::Track => self.track.id().to_string(),
            Column::Start => clock(self.start),
            Column::End => clock(self.end),
            Column::Duration => dialect.decimal(self.duration, 2),
            Column::Verdict => String::new(),
            Column::Hits => self.track.points().len().to_string(),
            Column::Chord => dialect.decimal(self.track.chord(), 1),
            Column::Path => dialect.decimal(self.path, 1),
            Column::Speed => dialect.decimal(self.path / self.duration, 1),
            Column::MaxArea => (self.track.max_area().max(0.0).round() as u64).to_string(),
            Column::Amplitude => dialect.decimal(self.track.peak_amplitude(), 1),
            Column::FilledFrames => self.filled.to_string(),
            Column::StartSeconds => dialect.decimal(self.start, 2),
        }
    }
}
