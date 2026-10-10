//! Background median, residual and per-pixel threshold of one target frame, fused, emitting
//! only the pixels above their threshold.

use crate::network::Network;

pub const LANES: usize = 64;
pub const MAD_TO_SIGMA: f32 = 1.4826;
pub const MAX_SAMPLES: usize = 32;

pub struct Params<'a> {
    pub threshold: f32,
    pub noise_factor: f32,
    pub use_noise: bool,
    pub gain_offset: f32,
    pub gains: &'a [f32],
}

#[derive(Clone, Copy)]
pub struct Above {
    pub pixel: u32,
    pub magnitude: f32,
}

fn small_median(values: &mut [f32]) -> f32 {
    values.sort_unstable_by(f32::total_cmp);
    let n = values.len();
    (values[(n - 1) / 2] + values[n / 2]) / 2.0
}

fn pixel_threshold(samples: &[&[u16]], p: usize, background: f32, params: &Params) -> f32 {
    let n = samples.len();
    let mut dev = [0f32; MAX_SAMPLES];
    for i in 0..n {
        dev[i] = f32::from(samples[i][p]) - background - params.gains[i];
    }
    let mut sorted = dev;
    let center = small_median(&mut sorted[..n]);
    let mut spread = [0f32; MAX_SAMPLES];
    for i in 0..n {
        spread[i] = (dev[i] - center).abs();
    }
    params.threshold.max(params.noise_factor * MAD_TO_SIGMA * small_median(&mut spread[..n]))
}

pub fn above_threshold(samples: &[&[u16]], current: &[u16], net: &Network, params: &Params, out: &mut Vec<Above>) {
    let n = samples.len();
    assert!(n <= MAX_SAMPLES && params.gains.len() == n);
    out.clear();
    let total = current.len();
    let mut lanes = [[0u16; LANES]; MAX_SAMPLES];
    let mut start = 0;
    while start < total {
        let len = LANES.min(total - start);
        for (lane, s) in lanes.iter_mut().zip(samples) {
            lane[..len].copy_from_slice(&s[start..start + len]);
        }
        for &(i, j) in &net.pairs {
            let (a, b) = lanes.split_at_mut(j);
            let (x, y) = (&mut a[i], &mut b[0]);
            for k in 0..LANES {
                let (p, q) = (x[k], y[k]);
                x[k] = p.min(q);
                y[k] = p.max(q);
            }
        }
        let mut residual = [0f32; LANES];
        let mut flagged = 0u64;
        for k in 0..len {
            let background = (f32::from(lanes[net.lo][k]) + f32::from(lanes[net.hi][k])) * 0.5;
            residual[k] = f32::from(current[start + k]) - background - params.gain_offset;
            flagged |= u64::from(residual[k].abs() > params.threshold) << k;
        }
        while flagged != 0 {
            let k = flagged.trailing_zeros() as usize;
            flagged &= flagged - 1;
            let p = start + k;
            let magnitude = residual[k].abs();
            let keep = if params.use_noise {
                let background = (f32::from(lanes[net.lo][k]) + f32::from(lanes[net.hi][k])) * 0.5;
                magnitude > pixel_threshold(samples, p, background, params)
            } else {
                true
            };
            if keep {
                out.push(Above { pixel: p as u32, magnitude });
            }
        }
        start += len;
    }
}
