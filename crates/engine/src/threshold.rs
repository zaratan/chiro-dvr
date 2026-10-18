use crate::config::DetectConfig;
use crate::convert::{narrowed, numbered};
use crate::mask::Mask;
use crate::network::Network;
use crate::prepare::to_units;

#[cfg(test)]
mod tests;

pub(crate) const LANES: usize = 64;
pub(crate) const MIN_NOISE_SAMPLES: usize = 7;
pub(crate) const STACKED_SAMPLES: usize = 64;
const MAD_TO_SIGMA: f32 = 1.4826;
pub(crate) const WEAK_RATIO: f32 = 0.75;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Lit {
    pub(crate) pixel: u32,
    pub(crate) magnitude: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Threshold {
    floor: f32,
    noise_factor: f32,
}

#[derive(Debug, Default)]
pub(crate) struct Scratch {
    lanes: Vec<[u16; LANES]>,
    deviations: Vec<f32>,
    pub(crate) lit: Vec<Lit>,
    pub(crate) weak: Vec<Lit>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Frame<'a> {
    pub(crate) samples: &'a [&'a [u16]],
    pub(crate) current: &'a [u16],
    pub(crate) network: &'a Network,
    pub(crate) gains: Gains<'a>,
    pub(crate) mask: &'a Mask,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Gains<'a> {
    pub(crate) current: f32,
    pub(crate) samples: &'a [f32],
}

impl Threshold {
    pub(crate) fn new(config: &DetectConfig) -> Self {
        Self {
            floor: to_units(config.threshold()),
            noise_factor: narrowed(config.noise_factor()) * MAD_TO_SIGMA,
        }
    }

    pub(crate) fn lit(self, frame: Frame<'_>, scratch: &mut Scratch) {
        let count = frame.samples.len();
        let Scratch {
            lanes,
            deviations,
            lit,
            weak,
        } = scratch;
        lit.clear();
        weak.clear();
        if deviations.len() < count {
            deviations.resize(count, 0.0);
        }
        if count <= STACKED_SAMPLES {
            let mut stacked = [[0_u16; LANES]; STACKED_SAMPLES];
            self.light(&mut stacked[..count], frame, deviations, lit, weak);
        } else {
            if lanes.len() < count {
                lanes.resize(count, [0; LANES]);
            }
            self.light(&mut lanes[..count], frame, deviations, lit, weak);
        }
    }

    fn light(
        self,
        lanes: &mut [[u16; LANES]],
        frame: Frame<'_>,
        deviations: &mut [f32],
        lit: &mut Vec<Lit>,
        weak: &mut Vec<Lit>,
    ) {
        let Frame {
            samples,
            current,
            network,
            gains,
            mask,
        } = frame;
        let noisy = self.noise_factor > 0.0 && samples.len() >= MIN_NOISE_SAMPLES;
        for (start, block) in (0..).step_by(LANES).zip(current.chunks(LANES)) {
            if mask.covers_all(start..start + block.len()) {
                continue;
            }
            for (lane, sample) in lanes.iter_mut().zip(samples) {
                lane[..block.len()].copy_from_slice(&sample[start..start + block.len()]);
            }
            network.order(lanes);
            let (low, high) = (&lanes[network.low()], &lanes[network.high()]);
            for (offset, ((&value, &low), &high)) in block.iter().zip(low).zip(high).enumerate() {
                let background = f32::midpoint(f32::from(low), f32::from(high));
                let magnitude = (f32::from(value) - background - gains.current).abs();
                if magnitude > self.floor * WEAK_RATIO && !mask.covers(start + offset) {
                    let bar = if noisy {
                        self.raised(samples, start + offset, background, gains.samples, deviations)
                    } else {
                        self.floor
                    };
                    let found = Lit {
                        pixel: numbered(start + offset),
                        magnitude,
                    };
                    if magnitude > bar {
                        lit.push(found);
                    } else if magnitude > bar * WEAK_RATIO {
                        weak.push(found);
                    }
                }
            }
        }
    }

    fn raised(self, samples: &[&[u16]], pixel: usize, background: f32, gains: &[f32], deviations: &mut [f32]) -> f32 {
        let deviations = &mut deviations[..samples.len()];
        for ((deviation, sample), gain) in deviations.iter_mut().zip(samples).zip(gains) {
            *deviation = f32::from(sample[pixel]) - background - gain;
        }
        let center = median(deviations);
        for deviation in deviations.iter_mut() {
            *deviation = (*deviation - center).abs();
        }
        self.floor.max(self.noise_factor * median(deviations))
    }
}

fn median(values: &mut [f32]) -> f32 {
    values.sort_unstable_by(f32::total_cmp);
    let count = values.len();
    f32::midpoint(values[(count - 1) / 2], values[count / 2])
}
