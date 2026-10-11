use crate::geometry::Geometry;

#[cfg(test)]
mod tests;

const UNITS_PER_LEVEL: u32 = 64;
pub(crate) const KERNEL: [u32; 9] = [1, 18, 221, 991, 1634, 991, 221, 18, 1];
const REACH: usize = KERNEL.len() / 2;
const KERNEL_SUM: u32 = 4096;
const AREA_WEIGHTS: u16 = 9;
const DIVISOR: u32 = KERNEL_SUM * KERNEL_SUM / UNITS_PER_LEVEL;
pub(crate) const GRAY: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 15, 16, 17, 18, 19,
    20, 22, 23, 24, 25, 26, 27, 28, 30, 31, 32, 33, 34, 35, 37, 38, 39, 40, 41, 42, 44, 45, 46, 47, 48, 49, 51, 52, 53,
    54, 55, 56, 58, 59, 60, 61, 62, 63, 65, 66, 67, 68, 69, 70, 72, 73, 74, 75, 76, 77, 79, 80, 81, 82, 83, 84, 86, 87,
    88, 89, 90, 91, 93, 94, 95, 96, 97, 98, 100, 101, 102, 103, 104, 105, 107, 108, 109, 110, 111, 112, 113, 115, 116,
    117, 118, 119, 120, 122, 123, 124, 125, 126, 127, 129, 130, 131, 132, 133, 134, 136, 137, 138, 139, 140, 141, 143,
    144, 145, 146, 147, 148, 150, 151, 152, 153, 154, 155, 157, 158, 159, 160, 161, 162, 164, 165, 166, 167, 168, 169,
    171, 172, 173, 174, 175, 176, 178, 179, 180, 181, 182, 183, 185, 186, 187, 188, 189, 190, 192, 193, 194, 195, 196,
    197, 198, 200, 201, 202, 203, 204, 205, 207, 208, 209, 210, 211, 212, 214, 215, 216, 217, 218, 219, 221, 222, 223,
    224, 225, 226, 228, 229, 230, 231, 232, 233, 235, 236, 237, 238, 239, 240, 242, 243, 244, 245, 246, 247, 249, 250,
    251, 252, 253, 254, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];

#[derive(Debug)]
pub(crate) struct Prepare {
    geometry: Geometry,
    gray: Vec<u8>,
    rows: Vec<u32>,
    column: Vec<u32>,
}

impl Prepare {
    pub(crate) fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            gray: vec![0; geometry.pixels()],
            rows: vec![0; geometry.pixels()],
            column: vec![0; geometry.width()],
        }
    }

    pub(crate) fn run(&mut self, luma: &[u8], out: &mut [u16]) -> u64 {
        let width = self.geometry.width();
        reduce(luma, self.geometry.source_width(), width, &mut self.gray);
        for (source, row) in self.gray.chunks_exact(width).zip(self.rows.chunks_exact_mut(width)) {
            blur_row(source, row);
        }
        let mut sum = 0;
        for (y, line) in out.chunks_exact_mut(width).enumerate() {
            self.column.fill(0);
            for (tap, &weight) in KERNEL.iter().enumerate() {
                let source = reflect(y + tap, self.geometry.height());
                let row = &self.rows[source * width..][..width];
                for (total, &value) in self.column.iter_mut().zip(row) {
                    *total += weight * value;
                }
            }
            for (unit, &total) in line.iter_mut().zip(&self.column) {
                *unit = units(total);
                sum += u64::from(*unit);
            }
        }
        sum
    }
}

fn reduce(luma: &[u8], source_width: usize, width: usize, gray: &mut [u8]) {
    for (source, work) in luma
        .chunks_exact(3 * source_width)
        .zip(gray.chunks_exact_mut(2 * width))
    {
        let (first, rest) = source.split_at(source_width);
        let (second, third) = rest.split_at(source_width);
        let (top, bottom) = work.split_at_mut(width);
        let thirds = first
            .as_chunks()
            .0
            .iter()
            .zip(second.as_chunks().0)
            .zip(third.as_chunks().0);
        let pairs = top.as_chunks_mut().0.iter_mut().zip(bottom.as_chunks_mut().0);
        for ((top, bottom), ((first, second), third)) in pairs.zip(thirds) {
            let (first, second, third) = (halves(*first), halves(*second), halves(*third));
            *top = [2 * first[0] + second[0], 2 * first[1] + second[1]].map(rounded);
            *bottom = [second[0] + 2 * third[0], second[1] + 2 * third[1]].map(rounded);
        }
    }
}

fn halves(three: [u8; 3]) -> [u16; 2] {
    let [left, middle, right] = three.map(|luma| u16::from(GRAY[usize::from(luma)]));
    [2 * left + middle, middle + 2 * right]
}

fn rounded(area: u16) -> u8 {
    ((area + AREA_WEIGHTS / 2) / AREA_WEIGHTS) as u8
}

fn blur_row(source: &[u8], row: &mut [u32]) {
    let width = source.len();
    for (out, window) in row[REACH..width - REACH].iter_mut().zip(source.windows(KERNEL.len())) {
        *out = KERNEL
            .iter()
            .zip(window)
            .map(|(&weight, &value)| weight * u32::from(value))
            .sum();
    }
    for x in (0..REACH).chain(width - REACH..width) {
        row[x] = KERNEL
            .iter()
            .enumerate()
            .map(|(tap, &weight)| weight * u32::from(source[reflect(x + tap, width)]))
            .sum();
    }
}

fn reflect(shifted: usize, length: usize) -> usize {
    let position = shifted.abs_diff(REACH);
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
