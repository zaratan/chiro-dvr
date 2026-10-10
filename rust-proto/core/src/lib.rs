pub mod frame;
pub mod pipeline;

use rayon::prelude::*;

pub const MAD_TO_SIGMA: f64 = 1.4826;
const BLOCK: usize = 512;
const MAX_SAMPLES: usize = 64;

pub trait Sample: Copy + Send + Sync {
    fn to_f64(self) -> f64;
    fn lo(a: Self, b: Self) -> Self;
    fn hi(a: Self, b: Self) -> Self;
}

impl Sample for f32 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
    fn lo(a: Self, b: Self) -> Self {
        a.min(b)
    }
    fn hi(a: Self, b: Self) -> Self {
        a.max(b)
    }
}

impl Sample for u8 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
    fn lo(a: Self, b: Self) -> Self {
        a.min(b)
    }
    fn hi(a: Self, b: Self) -> Self {
        a.max(b)
    }
}

/// Median of `values[..n]` with the same partial bubble network as `median.py`, applied lane by lane.
fn median_lanes<T: Sample>(values: &mut [[T; BLOCK]], n: usize, len: usize, out: &mut [f64]) {
    let (lo, hi) = ((n - 1) / 2, n / 2);
    for done in 0..(n - lo) {
        for j in 0..(n - 1 - done) {
            let (a, b) = values.split_at_mut(j + 1);
            let (x, y) = (&mut a[j], &mut b[0]);
            for k in 0..len {
                let (p, q) = (x[k], y[k]);
                x[k] = T::lo(p, q);
                y[k] = T::hi(p, q);
            }
        }
    }
    for k in 0..len {
        out[k] = (values[lo][k].to_f64() + values[hi][k].to_f64()) / 2.0;
    }
}

pub fn temporal_median<T: Sample + Default>(frames: &[&[T]], out: &mut [f64], parallel: bool) {
    let n = frames.len();
    assert!(n > 0 && n <= MAX_SAMPLES);
    assert!(frames.iter().all(|f| f.len() == out.len()));
    let work = |(b, chunk): (usize, &mut [f64])| {
        let start = b * BLOCK;
        let len = chunk.len();
        let mut values = vec![[T::default(); BLOCK]; n];
        for (v, f) in values.iter_mut().zip(frames) {
            v[..len].copy_from_slice(&f[start..start + len]);
        }
        median_lanes(&mut values, n, len, chunk);
    };
    if parallel {
        out.par_chunks_mut(BLOCK).enumerate().for_each(work);
    } else {
        out.chunks_mut(BLOCK).enumerate().for_each(work);
    }
}

fn median_small(values: &mut [f32]) -> f64 {
    let n = values.len();
    let (lo, hi) = ((n - 1) / 2, n / 2);
    for done in 0..(n - lo) {
        for j in 0..(n - 1 - done) {
            let (p, q) = (values[j], values[j + 1]);
            values[j] = p.min(q);
            values[j + 1] = p.max(q);
        }
    }
    (f64::from(values[lo]) + f64::from(values[hi])) / 2.0
}

/// `noise.temporal_noise` for one pixel: MAD of the gain-corrected deviations, in sigma units.
pub fn pixel_noise(samples: &[f32], background: f64, gains: &[f64]) -> f64 {
    let n = samples.len();
    let mut dev = [0f32; MAX_SAMPLES];
    for i in 0..n {
        dev[i] = ((f64::from(samples[i]) - background) - gains[i]) as f32;
    }
    let mut sorted = dev;
    let center = median_small(&mut sorted[..n]) as f32;
    let mut spread = [0f32; MAX_SAMPLES];
    for i in 0..n {
        spread[i] = (dev[i] - center).abs();
    }
    MAD_TO_SIGMA * median_small(&mut spread[..n])
}

pub struct FrameParams<'a> {
    pub gain_offset: f64,
    pub gains: &'a [f64],
    pub threshold: f64,
    pub noise_factor: f64,
    pub use_noise: bool,
}

/// Background, residual and per-pixel threshold of `detect_frames.process`, fused.
/// Returns true when some pixel is a candidate and `thresholds` holds a map.
pub fn residual_and_threshold(
    samples: &[&[f32]],
    current: &[f32],
    mask: &[bool],
    params: &FrameParams,
    residual: &mut [f64],
    thresholds: &mut [f64],
    parallel: bool,
) -> bool {
    let n = samples.len();
    assert!(n > 0 && n <= MAX_SAMPLES && params.gains.len() == n);
    let total = current.len();
    assert!(mask.len() == total && residual.len() == total && thresholds.len() == total);
    let work = |(b, (res, thr)): (usize, (&mut [f64], &mut [f64]))| -> bool {
        let start = b * BLOCK;
        let len = res.len();
        let mut values = vec![[0f32; BLOCK]; n];
        for (v, f) in values.iter_mut().zip(samples) {
            v[..len].copy_from_slice(&f[start..start + len]);
        }
        let mut background = [0f64; BLOCK];
        median_lanes(&mut values, n, len, &mut background);
        let mut any = false;
        let mut column = [0f32; MAX_SAMPLES];
        for k in 0..len {
            let p = start + k;
            let r = if mask[p] { (f64::from(current[p]) - background[k]) - params.gain_offset } else { 0.0 };
            res[k] = r;
            let mut t = params.threshold;
            if params.use_noise && r.abs() > params.threshold {
                any = true;
                for (i, f) in samples.iter().enumerate() {
                    column[i] = f[p];
                }
                let noise = pixel_noise(&column[..n], background[k], params.gains);
                t = params.threshold.max(params.noise_factor * noise);
            }
            thr[k] = t;
        }
        any
    };
    let chunks = residual.chunks_mut(BLOCK).zip(thresholds.chunks_mut(BLOCK)).enumerate();
    if parallel {
        residual
            .par_chunks_mut(BLOCK)
            .zip(thresholds.par_chunks_mut(BLOCK))
            .enumerate()
            .map(work)
            .reduce(|| false, |a, b| a | b)
    } else {
        chunks.map(work).fold(false, |a, b| a | b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_of_odd_count_is_the_middle_value() {
        let frames: Vec<Vec<f32>> = (0..11).map(|i| vec![(i * 7 % 11) as f32; 3]).collect();
        let refs: Vec<&[f32]> = frames.iter().map(|f| f.as_slice()).collect();
        let mut out = vec![0.0; 3];
        temporal_median(&refs, &mut out, false);
        assert_eq!(out, vec![5.0; 3]);
    }

    #[test]
    fn median_of_even_count_averages_the_two_middle_values_in_f64() {
        let frames: Vec<Vec<f32>> = [1.0f32, 2.0, 4.0, 8.0].iter().map(|v| vec![*v; 1]).collect();
        let refs: Vec<&[f32]> = frames.iter().map(|f| f.as_slice()).collect();
        let mut out = vec![0.0; 1];
        temporal_median(&refs, &mut out, false);
        assert_eq!(out, vec![3.0]);
    }

    #[test]
    fn parallel_and_sequential_medians_agree_across_block_edges() {
        let len = BLOCK * 3 + 17;
        let frames: Vec<Vec<f32>> =
            (0..9).map(|i| (0..len).map(|p| ((p * 31 + i * 17) % 97) as f32 * 0.37).collect()).collect();
        let refs: Vec<&[f32]> = frames.iter().map(|f| f.as_slice()).collect();
        let (mut a, mut b) = (vec![0.0; len], vec![0.0; len]);
        temporal_median(&refs, &mut a, false);
        temporal_median(&refs, &mut b, true);
        assert_eq!(a, b);
    }
}
