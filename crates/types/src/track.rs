use std::f64::consts::{PI, TAU};

use crate::detection::Detection;
use crate::frame::FrameIndex;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    id: usize,
    points: Vec<Detection>,
    part_starts: Vec<usize>,
}

/// A run of a track seen without a hiding in between: the whole track unless it was rejoined.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part<'a> {
    points: &'a [Detection],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("track {id} has no point")]
    Empty { id: usize },
    #[error("track {id} does not move forward in time at frame {frame}")]
    Unordered { id: usize, frame: FrameIndex },
}

impl Track {
    pub fn new(id: usize, points: Vec<Detection>) -> Result<Self, Error> {
        if points.is_empty() {
            return Err(Error::Empty { id });
        }
        if let Some(step) = points.windows(2).find(|step| step[1].frame <= step[0].frame) {
            return Err(Error::Unordered {
                id,
                frame: step[1].frame,
            });
        }
        Ok(Self {
            id,
            points,
            part_starts: Vec::new(),
        })
    }

    pub fn rejoined(id: usize, parts: Vec<Vec<Detection>>) -> Result<Self, Error> {
        if parts.iter().any(Vec::is_empty) {
            return Err(Error::Empty { id });
        }
        let mut part_starts = Vec::new();
        let mut points = Vec::new();
        for part in parts {
            if !points.is_empty() {
                part_starts.push(points.len());
            }
            points.extend(part);
        }
        Ok(Self {
            part_starts,
            ..Self::new(id, points)?
        })
    }

    #[must_use]
    pub fn with_id(self, id: usize) -> Self {
        Self { id, ..self }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn points(&self) -> &[Detection] {
        &self.points
    }

    pub fn first(&self) -> &Detection {
        &self.points[0]
    }

    pub fn last(&self) -> &Detection {
        &self.points[self.points.len() - 1]
    }

    pub fn chord(&self) -> f64 {
        self.first().distance(self.last())
    }

    pub fn path_length(&self) -> f64 {
        self.points
            .windows(2)
            .fold(0.0, |length, step| length + step[0].distance(&step[1]))
    }

    pub fn parts(&self) -> impl Iterator<Item = Part<'_>> {
        let starts = std::iter::once(0).chain(self.part_starts.iter().copied());
        let ends = self
            .part_starts
            .iter()
            .copied()
            .chain(std::iter::once(self.points.len()));
        starts.zip(ends).map(|(start, end)| Part {
            points: &self.points[start..end],
        })
    }

    pub fn max_area(&self) -> f64 {
        max_area(&self.points)
    }

    pub fn peak_amplitude(&self) -> f64 {
        peak_amplitude(&self.points)
    }

    pub fn median_turn(&self) -> f64 {
        let headings: Vec<f64> = self
            .points
            .windows(2)
            .filter(|step| (step[0].x, step[0].y) != (step[1].x, step[1].y))
            .map(|step| (step[1].y - step[0].y).atan2(step[1].x - step[0].x))
            .collect();
        let mut turns: Vec<f64> = headings
            .windows(2)
            .map(|change| ((change[1] - change[0] + PI).rem_euclid(TAU) - PI).abs())
            .collect();
        turns.sort_by(f64::total_cmp);
        median(&turns)
    }
}

impl<'a> Part<'a> {
    pub fn points(&self) -> &'a [Detection] {
        self.points
    }

    pub fn max_area(&self) -> f64 {
        max_area(self.points)
    }

    pub fn peak_amplitude(&self) -> f64 {
        peak_amplitude(self.points)
    }
}

fn max_area(points: &[Detection]) -> f64 {
    points.iter().map(|point| point.area).fold(f64::NEG_INFINITY, f64::max)
}

fn peak_amplitude(points: &[Detection]) -> f64 {
    points
        .iter()
        .map(|point| point.amplitude)
        .fold(f64::NEG_INFINITY, f64::max)
}

fn median(sorted: &[f64]) -> f64 {
    let middle = sorted.len() / 2;
    match sorted.len() {
        0 => 0.0,
        count if count % 2 == 1 => sorted[middle],
        _ => f64::midpoint(sorted[middle - 1], sorted[middle]),
    }
}
