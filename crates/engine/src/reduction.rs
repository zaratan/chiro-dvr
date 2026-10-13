use crate::geometry::Geometry;
use crate::gray::GRAY;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct Reduction {
    source_width: usize,
    width: usize,
    columns: Axis,
    rows: Axis,
    rounding: Rounding,
    source_gray: Vec<u8>,
    line: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rounding {
    Reciprocal { multiplier: u64, half: u64 },
    Exact { divisor: u64, half: u64 },
}

#[derive(Debug)]
struct Axis {
    phases: Vec<Vec<(usize, u32)>>,
    padded: Vec<(usize, u32)>,
    span: usize,
    weight: u64,
}

impl Reduction {
    pub(crate) fn new(geometry: Geometry) -> Self {
        Self::between(
            (geometry.source_width(), geometry.source_height()),
            (geometry.width(), geometry.height()),
        )
    }

    fn between((source_width, source_height): (usize, usize), (width, height): (usize, usize)) -> Self {
        let (columns, rows) = (Axis::new(source_width, width), Axis::new(source_height, height));
        Self {
            source_width,
            width,
            rounding: Rounding::new(columns.weight * rows.weight),
            columns,
            rows,
            source_gray: vec![0; source_width * source_height],
            line: vec![0; source_width],
        }
    }

    pub(crate) fn run(&mut self, luma: &[u8], gray: &mut [u8]) {
        for (converted, &value) in self.source_gray.iter_mut().zip(luma) {
            *converted = GRAY[usize::from(value)];
        }
        match self.rounding {
            reciprocal @ Rounding::Reciprocal { .. } => self.reduce(gray, move |total| reciprocal.rounded(total)),
            exact @ Rounding::Exact { .. } => self.reduce(gray, move |total| exact.rounded(total)),
        }
    }

    fn reduce(&mut self, gray: &mut [u8], rounded: impl Fn(u64) -> u8) {
        for (y, out) in gray.chunks_exact_mut(self.width).enumerate() {
            self.line.fill(0);
            for (row, weight) in self.rows.taps(y) {
                let source = &self.source_gray[row * self.source_width..][..self.source_width];
                for (total, &value) in self.line.iter_mut().zip(source) {
                    *total += weight * u32::from(value);
                }
            }
            let columns = &self.columns;
            match columns.padded.len() / columns.phases.len() {
                1 => across::<1>(&self.line, out, columns, &rounded),
                2 => across::<2>(&self.line, out, columns, &rounded),
                3 => across::<3>(&self.line, out, columns, &rounded),
                4 => across::<4>(&self.line, out, columns, &rounded),
                widest => across_any(&self.line, out, columns, widest, &rounded),
            }
        }
    }
}

fn across<const TAPS: usize>(line: &[u32], out: &mut [u8], columns: &Axis, rounded: &impl Fn(u64) -> u8) {
    let pattern = columns.padded.as_chunks::<TAPS>().0;
    for (block, values) in line.chunks_exact(columns.span).zip(out.chunks_exact_mut(pattern.len())) {
        for (value, taps) in values.iter_mut().zip(pattern) {
            let total: u64 = taps
                .iter()
                .map(|&(pixel, weight)| u64::from(weight) * u64::from(block[pixel]))
                .sum();
            *value = rounded(total);
        }
    }
}

fn across_any(line: &[u32], out: &mut [u8], columns: &Axis, widest: usize, rounded: &impl Fn(u64) -> u8) {
    let phases = columns.phases.len();
    for (block, values) in line.chunks_exact(columns.span).zip(out.chunks_exact_mut(phases)) {
        for (value, taps) in values.iter_mut().zip(columns.padded.chunks_exact(widest)) {
            let total: u64 = taps
                .iter()
                .map(|&(pixel, weight)| u64::from(weight) * u64::from(block[pixel]))
                .sum();
            *value = rounded(total);
        }
    }
}

impl Rounding {
    fn new(divisor: u64) -> Self {
        let half = divisor / 2;
        if divisor > 1 && u32::try_from(divisor * u64::from(u8::MAX) + half).is_ok() {
            Self::Reciprocal {
                multiplier: u64::MAX / divisor + 1,
                half,
            }
        } else {
            Self::Exact { divisor, half }
        }
    }

    fn rounded(self, total: u64) -> u8 {
        match self {
            Self::Reciprocal { multiplier, half } => ((u128::from(multiplier) * u128::from(total + half)) >> 64) as u8,
            Self::Exact { divisor, half } => ((total + half) / divisor) as u8,
        }
    }
}

impl Axis {
    fn new(source: usize, work: usize) -> Self {
        let common = greatest_common_divisor(source, work);
        let (span, unit) = (source / common, work / common);
        let phases: Vec<Vec<(usize, u32)>> = (0..unit)
            .map(|index| {
                let (start, end) = (index * span, (index + 1) * span);
                (start / unit..=(end - 1) / unit)
                    .map(|pixel| {
                        let overlap = ((pixel + 1) * unit).min(end) - (pixel * unit).max(start);
                        (pixel, weight(overlap))
                    })
                    .collect()
            })
            .collect();
        let widest = phases.iter().map(Vec::len).max().unwrap_or(0);
        let padded = phases
            .iter()
            .flat_map(|taps| taps.iter().copied().chain(std::iter::repeat((0, 0))).take(widest))
            .collect();
        Self {
            phases,
            padded,
            span,
            weight: span as u64,
        }
    }

    fn taps(&self, index: usize) -> impl Iterator<Item = (usize, u32)> + '_ {
        let base = index / self.phases.len() * self.span;
        self.phases[index % self.phases.len()]
            .iter()
            .map(move |&(pixel, weight)| (base + pixel, weight))
    }
}

fn weight(overlap: usize) -> u32 {
    overlap as u32
}

fn greatest_common_divisor(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
