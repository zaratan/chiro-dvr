use std::num::NonZeroUsize;
use std::ops::Range;

use crate::settings::Region;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("the OSD regions mask the whole {width}×{height} work frame")]
    WholeFrame { width: usize, height: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Mask {
    width: usize,
    rows: Vec<Vec<Range<usize>>>,
    kept: NonZeroUsize,
}

impl Mask {
    pub(crate) fn new(regions: &[Region], width: usize, height: usize, margin: usize) -> Result<Self, Error> {
        let mut rows: Vec<Vec<Range<usize>>> = Vec::new();
        if !regions.is_empty() {
            rows.resize(height, Vec::new());
            for &region in regions {
                let columns = region.columns(width, margin);
                for row in &mut rows[region.rows(height, margin)] {
                    row.push(columns.clone());
                }
            }
            rows.iter_mut().for_each(merge);
        }
        let masked: usize = rows.iter().flatten().map(ExactSizeIterator::len).sum();
        NonZeroUsize::new(width * height - masked)
            .map(|kept| Self { width, rows, kept })
            .ok_or(Error::WholeFrame { width, height })
    }

    pub(crate) fn kept(&self) -> NonZeroUsize {
        self.kept
    }

    pub(crate) fn covers(&self, pixel: usize) -> bool {
        self.rows
            .get(pixel / self.width)
            .is_some_and(|row| row.iter().any(|columns| columns.contains(&(pixel % self.width))))
    }

    pub(crate) fn masked_sum(&self, row: usize, line: &[u16]) -> u64 {
        self.rows.get(row).map_or(0, |spans| {
            spans
                .iter()
                .flat_map(|columns| &line[columns.clone()])
                .map(|&unit| u64::from(unit))
                .sum()
        })
    }
}

fn merge(spans: &mut Vec<Range<usize>>) {
    spans.sort_unstable_by_key(|columns| columns.start);
    spans.dedup_by(|next, last| {
        let overlapping = next.start <= last.end;
        if overlapping {
            last.end = last.end.max(next.end);
        }
        overlapping
    });
}
