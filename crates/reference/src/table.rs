use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

use crate::error::Error;

#[cfg(test)]
mod tests;

const HEADER: [&str; 12] = [
    "id",
    "start",
    "end",
    "start_s",
    "duration_s",
    "hits",
    "chord_px",
    "path_px",
    "speed_px_s",
    "max_area_px",
    "peak_amplitude",
    "filled_frames",
];

#[derive(Debug, Clone, PartialEq)]
pub struct TrackRow {
    pub id: usize,
    pub start: String,
    pub end: String,
    pub start_s: f64,
    pub duration_s: f64,
    pub hits: usize,
    pub chord_px: f64,
    pub path_px: f64,
    pub speed_px_s: f64,
    pub max_area_px: u64,
    pub peak_amplitude: f64,
    pub filled_frames: usize,
}

pub fn read(path: &Path) -> Result<Vec<TrackRow>, Error> {
    let text = fs::read_to_string(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut lines = text.lines();
    if lines.next() != Some(HEADER.join(",").as_str()) {
        return Err(Error::CsvHeader {
            path: path.to_path_buf(),
        });
    }
    lines
        .enumerate()
        .map(|(index, line)| Cells { path, line: index + 2 }.row(line))
        .collect()
}

pub fn write(path: &Path, rows: &[TrackRow]) -> Result<(), Error> {
    let failed = |source| Error::Write {
        path: path.to_path_buf(),
        source,
    };
    let mut file = BufWriter::new(File::create(path).map_err(failed)?);
    write!(file, "{}\r\n", HEADER.join(",")).map_err(failed)?;
    for row in rows {
        write!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{}\r\n",
            row.id,
            row.start,
            row.end,
            decimal(row.start_s),
            decimal(row.duration_s),
            row.hits,
            decimal(row.chord_px),
            decimal(row.path_px),
            decimal(row.speed_px_s),
            row.max_area_px,
            decimal(row.peak_amplitude),
            row.filled_frames,
        )
        .map_err(failed)?;
    }
    file.flush().map_err(failed)
}

fn decimal(value: f64) -> String {
    let text = value.to_string();
    if value.is_finite() && !text.contains('.') {
        text + ".0"
    } else {
        text
    }
}

struct Cells<'a> {
    path: &'a Path,
    line: usize,
}

impl Cells<'_> {
    fn row(&self, line: &str) -> Result<TrackRow, Error> {
        let cells: Vec<&str> = line.split(',').collect();
        let &[
            id,
            start,
            end,
            start_s,
            duration_s,
            hits,
            chord_px,
            path_px,
            speed_px_s,
            max_area_px,
            peak_amplitude,
            filled_frames,
        ] = cells.as_slice()
        else {
            return Err(Error::CsvWidth {
                path: self.path.to_path_buf(),
                line: self.line,
                found: cells.len(),
                expected: HEADER.len(),
            });
        };
        Ok(TrackRow {
            id: self.integer(id, "id")?,
            start: start.to_owned(),
            end: end.to_owned(),
            start_s: self.number(start_s, "start_s")?,
            duration_s: self.number(duration_s, "duration_s")?,
            hits: self.integer(hits, "hits")?,
            chord_px: self.number(chord_px, "chord_px")?,
            path_px: self.number(path_px, "path_px")?,
            speed_px_s: self.number(speed_px_s, "speed_px_s")?,
            max_area_px: self.integer(max_area_px, "max_area_px")?,
            peak_amplitude: self.number(peak_amplitude, "peak_amplitude")?,
            filled_frames: self.integer(filled_frames, "filled_frames")?,
        })
    }

    fn integer<T: FromStr<Err = ParseIntError>>(&self, text: &str, column: &'static str) -> Result<T, Error> {
        text.parse().map_err(|source| Error::CsvInteger {
            path: self.path.to_path_buf(),
            line: self.line,
            column,
            source,
        })
    }

    fn number(&self, text: &str, column: &'static str) -> Result<f64, Error> {
        text.parse().map_err(|source| Error::CsvNumber {
            path: self.path.to_path_buf(),
            line: self.line,
            column,
            source,
        })
    }
}
