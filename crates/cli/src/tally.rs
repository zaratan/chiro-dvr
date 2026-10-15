use std::path::Path;

use files::previous::Kept;

use crate::jobs::Refused;
use crate::outcome::{Failure, Row, State};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Tally {
    pub rows: Vec<Row>,
    pub refused: Vec<Refused>,
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

    pub fn rows_in<'a>(&'a self, folder: &'a Path) -> impl Iterator<Item = &'a Row> {
        self.rows.iter().filter(move |row| root(row) == folder)
    }

    pub fn done(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(|row| matches!(row.state, State::Done { .. }))
    }

    pub fn too_many(&self) -> impl Iterator<Item = (&Row, usize, usize)> {
        self.rows.iter().filter_map(|row| match row.state {
            State::TooManyTracks { count, limit, .. } => Some((row, count, limit)),
            State::Done { .. } | State::Failed(_) => None,
        })
    }

    pub fn failed(&self) -> impl Iterator<Item = (&Row, &Failure)> {
        self.rows.iter().filter_map(|row| match &row.state {
            State::Failed(failure) => Some((row, failure)),
            State::Done { .. } | State::TooManyTracks { .. } => None,
        })
    }

    pub fn kept(&self) -> impl Iterator<Item = (&Row, &Kept)> {
        self.rows.iter().flat_map(|row| {
            let kept: &[Kept] = match &row.state {
                State::Done { kept, .. } | State::TooManyTracks { kept, .. } => kept,
                State::Failed(_) => &[],
            };
            kept.iter().map(move |table| (row, table))
        })
    }

    pub fn all_done(&self) -> bool {
        self.refused.is_empty() && self.done().count() == self.rows.len()
    }
}

fn root(row: &Row) -> &Path {
    row.job.dest.parent().unwrap_or(&row.job.dest)
}
