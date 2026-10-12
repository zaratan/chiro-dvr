use types::detection::Measures;
use types::scale::WorkScale;

use crate::config::DetectConfig;
use crate::convert::{numbered, real};
use crate::geometry::Geometry;
use crate::prepare::to_level;
use crate::threshold::Lit;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct Components {
    geometry: Geometry,
    min_area: f64,
    max_area: f64,
    labels: Vec<u32>,
    parents: Vec<u32>,
    order: Vec<Option<usize>>,
    stats: Vec<Stats>,
}

#[derive(Debug, Clone, Copy)]
struct Stats {
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    columns: usize,
    rows: usize,
    pixels: usize,
    hits: usize,
    peak: f32,
}

impl Components {
    pub(crate) fn new(geometry: Geometry, config: &DetectConfig) -> Self {
        Self {
            geometry,
            min_area: config.min_area(),
            max_area: config.max_area(),
            labels: vec![0; geometry.pixels()],
            parents: Vec::new(),
            order: Vec::new(),
            stats: Vec::new(),
        }
    }

    pub(crate) fn run(&mut self, closed: &[u32], lit: &[Lit], found: &mut Vec<Measures>) {
        self.label(closed);
        self.measure(closed, lit);
        let scale = self.geometry.scale();
        found.extend(
            self.stats
                .iter()
                .map(|stats| stats.measures(scale))
                .filter(|measures| (self.min_area..=self.max_area).contains(&measures.area)),
        );
    }

    fn label(&mut self, closed: &[u32]) {
        let Self {
            geometry,
            labels,
            parents,
            ..
        } = self;
        parents.clear();
        parents.push(0);
        for &pixel in closed {
            let mut label = 0;
            for neighbour in earlier_neighbours(*geometry, pixel) {
                match (label, labels[neighbour]) {
                    (_, 0) => {}
                    (0, other) => label = other,
                    (_, other) => join(parents, label, other),
                }
            }
            if label == 0 {
                label = numbered(parents.len());
                parents.push(label);
            }
            labels[pixel as usize] = label;
        }
    }

    fn measure(&mut self, closed: &[u32], lit: &[Lit]) {
        let Self {
            geometry,
            labels,
            parents,
            order,
            stats,
            ..
        } = self;
        order.clear();
        order.resize(parents.len(), None);
        stats.clear();
        for &pixel in closed {
            let (column, row) = (geometry.column(pixel), geometry.row(pixel));
            let root = root(parents, labels[pixel as usize]) as usize;
            let component = *order[root].get_or_insert_with(|| {
                stats.push(Stats::at(column, row));
                stats.len() - 1
            });
            stats[component].add(column, row);
            labels[pixel as usize] = numbered(component + 1);
        }
        for lit in lit {
            if let Some(component) = (labels[lit.pixel as usize] as usize).checked_sub(1) {
                stats[component].hit(lit.magnitude);
            }
        }
        for &pixel in closed {
            labels[pixel as usize] = 0;
        }
    }
}

impl Stats {
    fn at(column: usize, row: usize) -> Self {
        Self {
            left: column,
            top: row,
            right: column,
            bottom: row,
            columns: 0,
            rows: 0,
            pixels: 0,
            hits: 0,
            peak: 0.0,
        }
    }

    fn add(&mut self, column: usize, row: usize) {
        self.left = self.left.min(column);
        self.top = self.top.min(row);
        self.right = self.right.max(column);
        self.bottom = self.bottom.max(row);
        self.columns += column;
        self.rows += row;
        self.pixels += 1;
    }

    fn hit(&mut self, magnitude: f32) {
        self.hits += 1;
        self.peak = self.peak.max(magnitude);
    }

    fn measures(&self, scale: WorkScale) -> Measures {
        let pixels = real(self.pixels);
        Measures {
            x: scale.coordinate_to_source(real(self.columns) / pixels),
            y: scale.coordinate_to_source(real(self.rows) / pixels),
            left: scale.length_to_source(real(self.left)),
            top: scale.length_to_source(real(self.top)),
            width: scale.length_to_source(real(self.right - self.left + 1)),
            height: scale.length_to_source(real(self.bottom - self.top + 1)),
            area: scale.area_to_source(real(self.hits)),
            amplitude: to_level(self.peak),
        }
    }
}

fn earlier_neighbours(geometry: Geometry, pixel: u32) -> impl Iterator<Item = usize> {
    let (column, row, width) = (geometry.column(pixel), geometry.row(pixel), geometry.width());
    let above = row.checked_sub(1);
    let left = column.checked_sub(1);
    let right = Some(column + 1).filter(|&right| right < width);
    [
        above.zip(left),
        above.zip(Some(column)),
        above.zip(right),
        Some(row).zip(left),
    ]
    .into_iter()
    .flatten()
    .map(move |(row, column)| row * width + column)
}

fn root(parents: &mut [u32], mut label: u32) -> u32 {
    while parents[label as usize] != label {
        let grandparent = parents[parents[label as usize] as usize];
        parents[label as usize] = grandparent;
        label = grandparent;
    }
    label
}

fn join(parents: &mut [u32], first: u32, second: u32) {
    let (first, second) = (root(parents, first), root(parents, second));
    parents[first.max(second) as usize] = first.min(second);
}
