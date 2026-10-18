use crate::geometry::Geometry;
use crate::gray;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) enum Reduction {
    Same,
    Area(Box<Area>),
}

#[derive(Debug)]
pub(crate) struct Area {
    source_width: usize,
    width: usize,
    columns: Axis,
    rows: Axis,
    rounding: Rounding,
    blocks: Option<Blocks>,
    source_gray: Vec<u8>,
    line: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rounding {
    Reciprocal { multiplier: u64, half: u64 },
    Exact { divisor: u64, half: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Narrow {
    multiplier: u32,
    half: u32,
}

#[derive(Debug)]
enum Blocks {
    TwoToOne(Fixed<2, 1>),
    ThreeToOne(Fixed<3, 1>),
    ThreeToTwo(Fixed<3, 2>),
    FourToThree(Fixed<4, 3>),
}

#[derive(Debug)]
struct Fixed<const SPAN: usize, const PHASES: usize> {
    weights: [[u32; SPAN]; PHASES],
    narrow: Narrow,
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

    fn between(source: (usize, usize), work: (usize, usize)) -> Self {
        if source == work {
            Self::Same
        } else {
            Self::Area(Box::new(Area::between(source, work)))
        }
    }

    pub(crate) fn run(&mut self, luma: &[u8], gray: &mut [u8]) {
        match self {
            Self::Same => to_gray(luma, gray),
            Self::Area(area) => area.run(luma, gray),
        }
    }
}

impl Area {
    fn between((source_width, source_height): (usize, usize), (width, height): (usize, usize)) -> Self {
        let (columns, rows) = (Axis::new(source_width, width), Axis::new(source_height, height));
        let divisor = columns.weight * rows.weight;
        Self {
            source_width,
            width,
            rounding: Rounding::new(divisor),
            blocks: Blocks::new(&columns, divisor),
            columns,
            rows,
            source_gray: vec![0; source_width * source_height],
            line: vec![0; source_width],
        }
    }

    fn run(&mut self, luma: &[u8], gray: &mut [u8]) {
        to_gray(luma, &mut self.source_gray);
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
            match (&self.blocks, columns.padded.len() / columns.phases.len()) {
                (Some(blocks), _) => blocks.across(&self.line, out),
                (None, 1) => across::<1>(&self.line, out, columns, &rounded),
                (None, 2) => across::<2>(&self.line, out, columns, &rounded),
                (None, 3) => across::<3>(&self.line, out, columns, &rounded),
                (None, 4) => across::<4>(&self.line, out, columns, &rounded),
                (None, widest) => across_any(&self.line, out, columns, widest, &rounded),
            }
        }
    }
}

fn to_gray(luma: &[u8], gray: &mut [u8]) {
    for (converted, &value) in gray.iter_mut().zip(luma) {
        *converted = gray::computed(value);
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

impl Blocks {
    fn new(columns: &Axis, divisor: u64) -> Option<Self> {
        let narrow = Narrow::new(divisor)?;
        let phases = &columns.phases;
        match (columns.span, phases.len()) {
            (2, 1) => Some(Self::TwoToOne(Fixed::new(phases, narrow))),
            (3, 1) => Some(Self::ThreeToOne(Fixed::new(phases, narrow))),
            (3, 2) => Some(Self::ThreeToTwo(Fixed::new(phases, narrow))),
            (4, 3) => Some(Self::FourToThree(Fixed::new(phases, narrow))),
            _ => None,
        }
    }

    fn across(&self, line: &[u32], out: &mut [u8]) {
        match self {
            Self::TwoToOne(fixed) => fixed.across(line, out),
            Self::ThreeToOne(fixed) => fixed.across(line, out),
            Self::ThreeToTwo(fixed) => fixed.across(line, out),
            Self::FourToThree(fixed) => fixed.across(line, out),
        }
    }
}

impl<const SPAN: usize, const PHASES: usize> Fixed<SPAN, PHASES> {
    fn new(phases: &[Vec<(usize, u32)>], narrow: Narrow) -> Self {
        let weights = std::array::from_fn(|phase| {
            std::array::from_fn(|pixel| {
                phases[phase]
                    .iter()
                    .find(|&&(at, _)| at == pixel)
                    .map_or(0, |&(_, weight)| weight)
            })
        });
        Self { weights, narrow }
    }

    fn across(&self, line: &[u32], out: &mut [u8]) {
        for (block, values) in line.as_chunks::<SPAN>().0.iter().zip(out.as_chunks_mut::<PHASES>().0) {
            for (value, phase) in values.iter_mut().zip(&self.weights) {
                let total: u32 = phase.iter().zip(block).map(|(&weight, &pixel)| weight * pixel).sum();
                *value = self.narrow.rounded(total);
            }
        }
    }
}

impl Narrow {
    const SHIFT: u32 = 23;
    const LARGEST_DIVISOR: u64 = 128;

    fn new(divisor: u64) -> Option<Self> {
        (1..=Self::LARGEST_DIVISOR).contains(&divisor).then(|| Self {
            multiplier: ((1_u64 << Self::SHIFT) / divisor + 1) as u32,
            half: (divisor / 2) as u32,
        })
    }

    fn rounded(self, total: u32) -> u8 {
        (((total + self.half) * self.multiplier) >> Self::SHIFT) as u8
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
