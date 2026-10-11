pub(crate) struct Noise(u64);

impl Noise {
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }

    pub(crate) fn uniform(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1_u64 << 53) as f64
    }

    pub(crate) fn gaussian(&mut self) -> f64 {
        (0..12).map(|_| self.uniform()).sum::<f64>() - 6.0
    }
}

pub(crate) fn luma(gray: f64) -> u8 {
    (gray * 219.0 / 255.0 + 16.0).round().clamp(0.0, 255.0) as u8
}

pub(crate) fn background(width: usize, height: usize, gray: f64, sigma: f64, seed: u64) -> Vec<u8> {
    let mut noise = Noise::new(seed);
    (0..width * height)
        .map(|_| luma(gray + sigma * noise.gaussian()))
        .collect()
}

pub(crate) fn with_square(
    mut plane: Vec<u8>,
    width: usize,
    corner: (usize, usize),
    size: usize,
    delta: f64,
) -> Vec<u8> {
    let shift = delta * 219.0 / 255.0;
    for row in corner.1..corner.1 + size {
        for value in &mut plane[row * width + corner.0..row * width + corner.0 + size] {
            *value = (f64::from(*value) + shift).round().clamp(0.0, 255.0) as u8;
        }
    }
    plane
}
