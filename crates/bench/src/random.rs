use std::num::NonZeroU64;

#[cfg(test)]
mod tests;

const UNIT: f64 = 1.0 / (1_u64 << 53) as f64;

#[derive(Debug, Clone)]
pub(crate) struct SplitMix64(u64);

impl SplitMix64 {
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }
}

#[derive(Debug, Clone)]
pub struct Xoshiro256 {
    state: [u64; 4],
}

impl Xoshiro256 {
    pub fn seeded(seed: u64) -> Self {
        let mut mix = SplitMix64::new(seed);
        Self {
            state: [mix.next_u64(), mix.next_u64(), mix.next_u64(), mix.next_u64()],
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let [s0, s1, s2, s3] = &mut self.state;
        let result = s1.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let shifted = *s1 << 17;
        *s2 ^= *s0;
        *s3 ^= *s1;
        *s1 ^= *s2;
        *s0 ^= *s3;
        *s2 ^= shifted;
        *s3 = s3.rotate_left(45);
        result
    }

    pub fn uniform(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * ((self.next_u64() >> 11) as f64 * UNIT)
    }

    pub fn below(&mut self, bound: NonZeroU64) -> u64 {
        let last_accepted = u64::MAX - rejected(bound);
        loop {
            let drawn = self.next_u64();
            if drawn <= last_accepted {
                return drawn % bound.get();
            }
        }
    }

    pub fn sign(&mut self) -> f64 {
        if self.next_u64() >> 63 == 0 { -1.0 } else { 1.0 }
    }
}

fn rejected(bound: NonZeroU64) -> u64 {
    (u64::MAX % bound.get() + 1) % bound.get()
}
