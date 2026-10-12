use std::collections::HashMap;

use crate::geometry::{Spot, Trajectory, distance};

#[cfg(test)]
mod tests;

const CANDIDATE_STEP_PER_RADIUS: f64 = 0.5;
const THOUSANDTHS: f64 = 1000.0;
const PREFERRED_THOUSANDTHS: i64 = 250;
const CROSSING_WEIGHT: i64 = 1000;
const START_BIAS: i64 = 2;
const CROSSING_REACH: f64 = 2.0;
const MARKER_GAP: f64 = 2.0;
const REPAIR_ROUNDS: usize = 8;
const OFFSET_RINGS: usize = 40;
const OFFSET_STEP_PER_RADIUS: f64 = 0.5;
const OFFSET_DIRECTIONS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeepClear<'a> {
    pub spots: &'a [Spot],
    pub radius: f64,
}

pub fn clamp(point: Spot, frame: Frame, radius: f64) -> Spot {
    let inset = radius + MARKER_GAP;
    [
        point[0].max(inset).min(frame.width - inset),
        point[1].max(inset).min(frame.height - inset),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Candidate {
    thousandths: i64,
    on_path: Spot,
    center: Spot,
}

fn candidates(path: &Trajectory, frame: Frame, radius: f64) -> Vec<Candidate> {
    let total = path.total();
    let lengths = if total > 0.0 {
        path_lengths(total, radius * CANDIDATE_STEP_PER_RADIUS)
    } else {
        vec![0.0]
    };
    lengths
        .into_iter()
        .map(|length| {
            let on_path = path.at(length);
            Candidate {
                thousandths: if total > 0.0 {
                    (length / total * THOUSANDTHS).round() as i64
                } else {
                    0
                },
                on_path,
                center: clamp(on_path, frame, radius),
            }
        })
        .collect()
}

fn path_lengths(total: f64, step: f64) -> Vec<f64> {
    let steps = if step > 0.0 { (total / step).ceil() as usize } else { 0 };
    let mut lengths: Vec<f64> = (0..steps)
        .map(|index| index as f64 * step)
        .filter(|&length| length < total)
        .collect();
    lengths.push(total);
    lengths
}

struct Crossings {
    cell: f64,
    cells: HashMap<(i64, i64), Vec<(usize, Spot)>>,
}

impl Crossings {
    fn new(sampled: &[Vec<Spot>], reach: f64) -> Self {
        let cell = reach.max(1.0);
        let mut cells: HashMap<(i64, i64), Vec<(usize, Spot)>> = HashMap::new();
        for (owner, spots) in sampled.iter().enumerate() {
            for &spot in spots {
                cells.entry(key(spot, cell)).or_default().push((owner, spot));
            }
        }
        Self { cell, cells }
    }

    fn near(&self, center: Spot, owner: usize) -> i64 {
        let (column, row) = key(center, self.cell);
        let mut count = 0_i64;
        for dx in -1..=1 {
            for dy in -1..=1 {
                let Some(spots) = self.cells.get(&(column + dx, row + dy)) else {
                    continue;
                };
                count += spots
                    .iter()
                    .filter(|&&(other, spot)| other != owner && distance(center, spot) < self.cell)
                    .count() as i64;
            }
        }
        count
    }
}

fn key(spot: Spot, cell: f64) -> (i64, i64) {
    ((spot[0] / cell).floor() as i64, (spot[1] / cell).floor() as i64)
}

fn collisions(center: Spot, others: &[Spot], radius: f64, keep_clear: KeepClear<'_>) -> (usize, usize) {
    let markers = others
        .iter()
        .filter(|&&other| distance(center, other) < 2.0 * radius + MARKER_GAP)
        .count();
    let heads = keep_clear
        .spots
        .iter()
        .filter(|&&spot| distance(center, spot) < keep_clear.radius)
        .count();
    (markers, heads)
}

struct Placer<'a> {
    candidates: Vec<Vec<Candidate>>,
    crossings: Crossings,
    frame: Frame,
    radius: f64,
    keep_clear: KeepClear<'a>,
}

impl Placer<'_> {
    fn best(&self, index: usize, others: &[Spot]) -> Option<((usize, usize), Spot)> {
        let on_path = self.cheapest(index, others, self.candidates.get(index)?.iter().copied())?;
        if on_path.0.0 == 0 {
            return Some(on_path);
        }
        let beside = (1..=OFFSET_RINGS).find_map(|ring| {
            self.cheapest(index, others, self.ring(index, ring))
                .filter(|&((markers, _), _)| markers == 0)
        });
        Some(beside.unwrap_or(on_path))
    }

    fn ring(&self, index: usize, ring: usize) -> impl Iterator<Item = Candidate> + '_ {
        let offset = ring as f64 * OFFSET_STEP_PER_RADIUS * self.radius;
        self.candidates
            .get(index)
            .into_iter()
            .flatten()
            .flat_map(move |candidate| {
                (0..OFFSET_DIRECTIONS).map(move |direction| {
                    let angle = std::f64::consts::TAU * direction as f64 / OFFSET_DIRECTIONS as f64;
                    let moved = [
                        candidate.on_path[0] + offset * angle.cos(),
                        candidate.on_path[1] + offset * angle.sin(),
                    ];
                    Candidate {
                        center: clamp(moved, self.frame, self.radius),
                        ..*candidate
                    }
                })
            })
    }

    fn cheapest(
        &self,
        index: usize,
        others: &[Spot],
        candidates: impl Iterator<Item = Candidate>,
    ) -> Option<((usize, usize), Spot)> {
        candidates
            .map(|candidate| {
                let bias = START_BIAS * (candidate.thousandths - PREFERRED_THOUSANDTHS).abs();
                let cost = CROSSING_WEIGHT * self.crossings.near(candidate.center, index) + bias;
                let hits = collisions(candidate.center, others, self.radius, self.keep_clear);
                ((hits, cost, candidate.thousandths), candidate.center)
            })
            .min_by_key(|&(key, _)| key)
            .map(|((hits, _, _), center)| (hits, center))
    }
}

impl Placer<'_> {
    fn greedy(&self, order: &[usize]) -> Vec<Spot> {
        let mut chosen: Vec<Option<Spot>> = vec![None; self.candidates.len()];
        for &index in order {
            let others: Vec<Spot> = chosen.iter().flatten().copied().collect();
            chosen[index] = self.best(index, &others).map(|(_, center)| center);
        }
        chosen.into_iter().map(Option::unwrap_or_default).collect()
    }

    fn repair(&self, order: &[usize], centers: &mut [Spot]) {
        for _ in 0..REPAIR_ROUNDS {
            let mut moved = false;
            for &index in order {
                let others: Vec<Spot> = centers
                    .iter()
                    .enumerate()
                    .filter(|&(other, _)| other != index)
                    .map(|(_, &center)| center)
                    .collect();
                let current = collisions(centers[index], &others, self.radius, self.keep_clear);
                if current == (0, 0) {
                    continue;
                }
                if let Some((hits, center)) = self.best(index, &others)
                    && hits < current
                {
                    centers[index] = center;
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
    }
}

pub fn place_markers(paths: &[Trajectory], frame: Frame, radius: f64, keep_clear: KeepClear<'_>) -> Vec<Spot> {
    let sampled: Vec<Vec<Spot>> = paths.iter().map(|path| path.resampled(radius / 2.0)).collect();
    let placer = Placer {
        candidates: paths.iter().map(|path| candidates(path, frame, radius)).collect(),
        crossings: Crossings::new(&sampled, CROSSING_REACH * radius),
        frame,
        radius,
        keep_clear,
    };
    let mut order: Vec<usize> = (0..paths.len()).collect();
    order.sort_by(|&a, &b| paths[a].total().total_cmp(&paths[b].total()));
    let mut centers = placer.greedy(&order);
    placer.repair(&order, &mut centers);
    centers
}
