use std::fmt::Debug;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use tempfile::TempPath;
use types::language::Language;
use types::timeline::Timeline;
use types::track::Track;

use crate::dialect::Dialect;
use crate::error::Error;
use crate::exclusive;
use crate::names::partial_path;
use crate::time::{clock, frame_duration, frame_seconds};

#[cfg(test)]
mod tests;

const LINE_END: &str = "\r\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Track,
    Start,
    End,
    Duration,
    Hits,
    Chord,
    Path,
    Speed,
    MaxArea,
    Amplitude,
    FilledFrames,
    StartSeconds,
}

pub const COLUMNS: [Column; 12] = [
    Column::Track,
    Column::Start,
    Column::End,
    Column::Duration,
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
    exclusive::rename(partial, path).map_err(|source| match source.kind() {
        io::ErrorKind::AlreadyExists => Error::Taken {
            path: path.to_path_buf(),
        },
        _ => failed(source),
    })
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
