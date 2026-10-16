use std::path::Path;

use files::params::Finish;
use files::previous::Kept;

use crate::jobs::Refused;
use crate::outcome::{Failure, Row, State};
use crate::reuse::Earlier;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Tally {
    pub rows: Vec<Row>,
    pub refused: Vec<Refused>,
    pub unwritten: Vec<files::error::Error>,
}

impl Tally {
    pub fn roots(&self) -> Vec<&Path> {
        let mut roots: Vec<&Path> = Vec::new();
        for root in self.rows.iter().map(root) {
            if !roots.contains(&root) && self.exists(root) {
                roots.push(root);
            }
        }
        roots
    }

    fn exists(&self, folder: &Path) -> bool {
        self.rows_in(folder).any(|row| !matches!(row.state, State::Failed(_))) || folder.is_dir()
    }

    pub fn unwritten_files(&self) -> Vec<&files::error::Error> {
        let mut once: Vec<&files::error::Error> = Vec::new();
        for error in &self.unwritten {
            if !once.iter().any(|named| named.path() == error.path()) {
                once.push(error);
            }
        }
        once
    }

    pub fn rows_in<'a>(&'a self, folder: &'a Path) -> impl Iterator<Item = &'a Row> {
        self.rows.iter().filter(move |row| root(row) == folder)
    }

    pub fn done(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(|row| matches!(row.state, State::Done { .. }))
    }

    pub fn already_done(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(|row| {
            matches!(
                row.state,
                State::Skipped(Earlier {
                    finish: Finish::Done,
                    ..
                })
            )
        })
    }

    pub fn too_many(&self) -> impl Iterator<Item = (&Row, usize, usize)> {
        self.rows.iter().filter_map(|row| match row.state {
            State::TooManyTracks { count, limit, .. }
            | State::Skipped(Earlier {
                finish: Finish::TooManyTracks,
                tracks: count,
                limit,
                ..
            }) => Some((row, count, limit)),
            State::Done { .. } | State::Skipped(_) | State::Failed(_) => None,
        })
    }

    pub fn failed(&self) -> impl Iterator<Item = (&Row, &Failure)> {
        self.rows.iter().filter_map(|row| match &row.state {
            State::Failed(failure) => Some((row, failure)),
            State::Done { .. } | State::TooManyTracks { .. } | State::Skipped(_) => None,
        })
    }

    pub fn kept(&self) -> impl Iterator<Item = (&Row, &Kept)> {
        self.rows.iter().flat_map(|row| {
            let kept: &[Kept] = match &row.state {
                State::Done { kept, .. } | State::TooManyTracks { kept, .. } => kept,
                State::Skipped(_) | State::Failed(_) => &[],
            };
            kept.iter().map(move |table| (row, table))
        })
    }

    pub fn all_done(&self) -> bool {
        self.refused.is_empty()
            && self.unwritten.is_empty()
            && self.done().count() + self.already_done().count() == self.rows.len()
    }
}

fn root(row: &Row) -> &Path {
    row.job.dest.parent().unwrap_or(&row.job.dest)
}
