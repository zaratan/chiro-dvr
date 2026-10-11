use std::ops::RangeInclusive;

use crate::convert::numbered;
use crate::geometry::Geometry;
use crate::threshold::Lit;

#[cfg(test)]
mod tests;

pub(crate) const RADIUS: usize = 4;
pub(crate) const DISK: [usize; 2 * RADIUS + 1] = [0, 3, 3, 4, 4, 4, 3, 3, 0];

#[derive(Debug)]
pub(crate) struct Closing {
    geometry: Geometry,
    marks: Vec<bool>,
    touched: Vec<u32>,
    closed: Vec<u32>,
}

impl Closing {
    pub(crate) fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            marks: vec![false; geometry.pixels()],
            touched: Vec::new(),
            closed: Vec::new(),
        }
    }

    pub(crate) fn run(&mut self, lit: &[Lit]) -> &[u32] {
        let Self {
            geometry,
            marks,
            touched,
            closed,
        } = self;
        let width = geometry.width();
        touched.clear();
        closed.clear();
        for pixel in lit.iter().map(|lit| lit.pixel) {
            for (row, columns) in disk(*geometry, pixel) {
                for column in columns {
                    let around = row * width + column;
                    if !marks[around] {
                        marks[around] = true;
                        touched.push(numbered(around));
                    }
                }
            }
        }
        closed.extend(touched.iter().copied().filter(|&pixel| {
            disk(*geometry, pixel).all(|(row, columns)| marks[row * width..][columns].iter().all(|&mark| mark))
        }));
        closed.sort_unstable();
        for &pixel in touched.iter() {
            marks[pixel as usize] = false;
        }
        closed
    }
}

fn disk(geometry: Geometry, pixel: u32) -> impl Iterator<Item = (usize, RangeInclusive<usize>)> {
    let (column, row) = (geometry.column(pixel), geometry.row(pixel));
    let last_column = geometry.width() - 1;
    DISK.iter().enumerate().filter_map(move |(offset, &half)| {
        let around = (row + offset)
            .checked_sub(RADIUS)
            .filter(|&around| around < geometry.height())?;
        Some((around, column.saturating_sub(half)..=(column + half).min(last_column)))
    })
}
