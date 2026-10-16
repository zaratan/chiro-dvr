use std::fs;
use std::path::Path;

use crate::params::TrackEntry;
use crate::table::{Column, Headings, LANGUAGES};

#[cfg(test)]
mod tests;

const SEPARATORS: [char; 3] = [';', ',', '\t'];
const BYTE_ORDER_MARK: char = '\u{feff}';
const QUOTE: char = '"';
const SAME_START_S: f64 = 0.006;

pub fn read_verdicts(path: &Path, tracks: &[TrackEntry], headings: &dyn Headings) -> Vec<Option<String>> {
    let Ok(bytes) = fs::read(path) else {
        return vec![None; tracks.len()];
    };
    let text = str::from_utf8(&bytes).map_or_else(|_| bytes.iter().copied().map(char::from).collect(), str::to_owned);
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let Some(header) = lines.next().map(|line| line.trim_start_matches(BYTE_ORDER_MARK)) else {
        return vec![None; tracks.len()];
    };
    let Some(columns) = Columns::find(header, headings) else {
        return vec![None; tracks.len()];
    };
    let rows: Vec<(usize, f64, String)> = lines.filter_map(|line| columns.read(line)).collect();
    tracks
        .iter()
        .map(|track| {
            rows.iter()
                .find(|(id, start_s, _)| *id == track.id && (start_s - track.start_s).abs() < SAME_START_S)
                .map(|(_, _, verdict)| verdict.trim().to_owned())
                .filter(|verdict| !verdict.is_empty())
        })
        .collect()
}

#[derive(Debug)]
struct Columns {
    separator: char,
    track: usize,
    start_s: usize,
    verdict: usize,
}

impl Columns {
    fn find(header: &str, headings: &dyn Headings) -> Option<Self> {
        SEPARATORS.into_iter().find_map(|separator| {
            let cells: Vec<String> = cells(header, separator)
                .iter()
                .map(|cell| cell.trim().to_lowercase())
                .collect();
            let position = |column: Column| {
                LANGUAGES.iter().find_map(|&language| {
                    let heading = headings.heading(column, language).to_lowercase();
                    cells.iter().position(|cell| *cell == heading)
                })
            };
            Some(Self {
                separator,
                track: position(Column::Track)?,
                start_s: position(Column::StartSeconds)?,
                verdict: position(Column::Verdict)?,
            })
        })
    }

    fn read(&self, line: &str) -> Option<(usize, f64, String)> {
        let cells = cells(line, self.separator);
        let id = cells.get(self.track)?.trim().parse().ok()?;
        let start_s = cells.get(self.start_s)?.trim().replace(',', ".").parse().ok()?;
        Some((id, start_s, cells.get(self.verdict).cloned().unwrap_or_default()))
    }
}

fn cells(line: &str, separator: char) -> Vec<String> {
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut characters = line.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            QUOTE if quoted && characters.peek() == Some(&QUOTE) => {
                cell.push(QUOTE);
                characters.next();
            }
            QUOTE => quoted = !quoted,
            other if other == separator && !quoted => cells.push(std::mem::take(&mut cell)),
            other => cell.push(other),
        }
    }
    cells.push(cell);
    cells
}
