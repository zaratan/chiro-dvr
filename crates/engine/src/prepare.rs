use crate::blur::{Blur, KERNEL_SUM};
use crate::geometry::Geometry;
use crate::mask::Mask;
use crate::reduction::Reduction;

#[cfg(test)]
mod tests;

const UNITS_PER_LEVEL: u32 = 64;
const DIVISOR: u32 = KERNEL_SUM * KERNEL_SUM / UNITS_PER_LEVEL;

#[derive(Debug)]
pub(crate) struct Prepare {
    geometry: Geometry,
    blur: Blur,
    reduction: Reduction,
    gray: Vec<u8>,
    rows: Vec<u32>,
    column: Vec<u32>,
}

impl Prepare {
    pub(crate) fn new(geometry: Geometry, blur: &Blur) -> Self {
        Self {
            geometry,
            blur: blur.clone(),
            reduction: Reduction::new(geometry),
            gray: vec![0; geometry.pixels()],
            rows: vec![0; geometry.pixels()],
            column: vec![0; geometry.width()],
        }
    }

    pub(crate) fn run(&mut self, luma: &[u8], mask: &Mask, out: &mut [u16]) -> u64 {
        let width = self.geometry.width();
        self.reduction.run(luma, &mut self.gray);
        let (weights, reach) = (self.blur.weights(), self.blur.reach());
        for (source, row) in self.gray.chunks_exact(width).zip(self.rows.chunks_exact_mut(width)) {
            blur_row(weights, source, row);
        }
        let mut sum = 0;
        for (y, line) in out.chunks_exact_mut(width).enumerate() {
            self.column.fill(0);
            for (tap, &weight) in weights.iter().enumerate() {
                let source = reflect(y + tap, reach, self.geometry.height());
                let row = &self.rows[source * width..][..width];
                for (total, &value) in self.column.iter_mut().zip(row) {
                    *total += weight * value;
                }
            }
            for (unit, &total) in line.iter_mut().zip(&self.column) {
                *unit = units(total);
                sum += u64::from(*unit);
            }
            sum -= mask.masked_sum(y, line);
        }
        sum
    }
}

fn blur_row(weights: &[u32], source: &[u8], row: &mut [u32]) {
    if let Ok(kernel) = <&[u32; 1]>::try_from(weights) {
        blur_row_by(kernel, source, row);
    } else if let Ok(kernel) = <&[u32; 3]>::try_from(weights) {
        blur_row_by(kernel, source, row);
    } else if let Ok(kernel) = <&[u32; 5]>::try_from(weights) {
        blur_row_by(kernel, source, row);
    } else if let Ok(kernel) = <&[u32; 7]>::try_from(weights) {
        blur_row_by(kernel, source, row);
    } else if let Ok(kernel) = <&[u32; 9]>::try_from(weights) {
        blur_row_by(kernel, source, row);
    } else if let Ok(kernel) = <&[u32; 11]>::try_from(weights) {
        blur_row_by(kernel, source, row);
    } else if let Ok(kernel) = <&[u32; 13]>::try_from(weights) {
        blur_row_by(kernel, source, row);
    } else if let Ok(kernel) = <&[u32; 15]>::try_from(weights) {
        blur_row_by(kernel, source, row);
    } else if let Ok(kernel) = <&[u32; 17]>::try_from(weights) {
        blur_row_by(kernel, source, row);
    } else {
        blur_row_any(weights, source, row);
    }
}

fn blur_row_by<const TAPS: usize>(kernel: &[u32; TAPS], source: &[u8], row: &mut [u32]) {
    let (width, reach) = (source.len(), TAPS / 2);
    for (out, window) in row[reach..width - reach].iter_mut().zip(source.windows(TAPS)) {
        *out = kernel
            .iter()
            .zip(window)
            .map(|(&weight, &value)| weight * u32::from(value))
            .sum();
    }
    blur_edges(kernel, source, row);
}

fn blur_row_any(weights: &[u32], source: &[u8], row: &mut [u32]) {
    let (width, reach) = (source.len(), weights.len() / 2);
    let inner = &mut row[reach..width - reach];
    inner.fill(0);
    for (tap, &weight) in weights.iter().enumerate() {
        for (out, &value) in inner.iter_mut().zip(&source[tap..]) {
            *out += weight * u32::from(value);
        }
    }
    blur_edges(weights, source, row);
}

fn blur_edges(weights: &[u32], source: &[u8], row: &mut [u32]) {
    let (width, reach) = (source.len(), weights.len() / 2);
    for x in (0..reach).chain(width - reach..width) {
        row[x] = weights
            .iter()
            .enumerate()
            .map(|(tap, &weight)| weight * u32::from(source[reflect(x + tap, reach, width)]))
            .sum();
    }
}

fn reflect(shifted: usize, reach: usize, length: usize) -> usize {
    let position = shifted.abs_diff(reach);
    if position < length {
        position
    } else {
        2 * (length - 1) - position
    }
}

pub(crate) fn to_units(level: f64) -> f32 {
    (level * f64::from(UNITS_PER_LEVEL)) as f32
}

pub(crate) fn to_level(units: f32) -> f64 {
    f64::from(units) / f64::from(UNITS_PER_LEVEL)
}

fn units(total: u32) -> u16 {
    ((total + DIVISOR / 2) / DIVISOR) as u16
}
