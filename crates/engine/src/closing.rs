use std::ops::RangeInclusive;

use crate::convert::{numbered, real};
use crate::geometry::Geometry;
use crate::threshold::Lit;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Disk {
    halves: Vec<usize>,
}

impl Disk {
    pub(crate) fn new(radius: f64, geometry: Geometry) -> Self {
        let (lowest_row, last_column) = (geometry.height() - 1, geometry.width() - 1);
        let radius = whole(radius.round().min(real(lowest_row + last_column)));
        let kept = radius.min(lowest_row);
        let halves = (0..=2 * kept)
            .map(|row| whole(half_width(radius, row.abs_diff(kept))).min(last_column))
            .collect();
        Self { halves }
    }

    fn reach(&self) -> usize {
        self.halves.len() / 2
    }
}

fn half_width(radius: usize, offset: usize) -> f64 {
    (real(radius - offset) * real(radius + offset)).sqrt().round()
}

fn whole(count: f64) -> usize {
    count as usize
}

#[derive(Debug)]
pub(crate) struct Closing {
    geometry: Geometry,
    disk: Disk,
    marks: Vec<bool>,
    touched: Vec<u32>,
    closed: Vec<u32>,
}

impl Closing {
    pub(crate) fn new(geometry: Geometry, disk: &Disk) -> Self {
        Self {
            geometry,
            disk: disk.clone(),
            marks: vec![false; geometry.pixels()],
            touched: Vec::new(),
            closed: Vec::new(),
        }
    }

    pub(crate) fn run(&mut self, lit: &[Lit]) -> &[u32] {
        let Self {
            geometry,
            disk,
            marks,
            touched,
            closed,
        } = self;
        let width = geometry.width();
        touched.clear();
        closed.clear();
        for pixel in lit.iter().map(|lit| lit.pixel) {
            for (row, columns) in covered(*geometry, disk, pixel) {
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
            covered(*geometry, disk, pixel).all(|(row, columns)| marks[row * width..][columns].iter().all(|&mark| mark))
        }));
        closed.sort_unstable();
        for &pixel in touched.iter() {
            marks[pixel as usize] = false;
        }
        closed
    }
}

fn covered(geometry: Geometry, disk: &Disk, pixel: u32) -> impl Iterator<Item = (usize, RangeInclusive<usize>)> {
    let (column, row) = (geometry.column(pixel), geometry.row(pixel));
    let (last_column, reach) = (geometry.width() - 1, disk.reach());
    disk.halves.iter().enumerate().filter_map(move |(offset, &half)| {
        let around = (row + offset)
            .checked_sub(reach)
            .filter(|&around| around < geometry.height())?;
        Some((around, column.saturating_sub(half)..=(column + half).min(last_column)))
    })
}
