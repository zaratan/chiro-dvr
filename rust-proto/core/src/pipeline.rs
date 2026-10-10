use std::collections::VecDeque;

use crate::frame::{Blob, BlobConfig, find_blobs_sparse};
use crate::{FrameParams, residual_and_threshold};

pub const MIN_NOISE_SAMPLES: usize = 7;

pub struct DetectConfig {
    pub threshold: f64,
    pub min_area: f64,
    pub max_area: f64,
    pub bg_window_s: f64,
    pub bg_step: usize,
    pub merge_radius: f64,
    pub work_width: usize,
    pub noise_factor: f64,
    pub target_sigma: f64,
}

impl Default for DetectConfig {
    fn default() -> Self {
        Self {
            threshold: 12.0,
            min_area: 4.0,
            max_area: 2700.0,
            bg_window_s: 1.0,
            bg_step: 3,
            merge_radius: 6.0,
            work_width: 960,
            noise_factor: 8.0,
            target_sigma: 1.5,
        }
    }
}

impl DetectConfig {
    pub fn half_window(&self, fps: f64) -> usize {
        (self.bg_window_s * fps / 2.0).round_ties_even() as usize
    }
}

/// numpy's `np.mean` of a short list of Python floats (float64 pairwise summation).
pub fn mean_f64(a: &[f64]) -> f64 {
    let n = a.len();
    let sum = if n < 8 {
        a.iter().fold(-0.0, |s, v| s + v)
    } else {
        let mut r = [0f64; 8];
        r.copy_from_slice(&a[..8]);
        let mut i = 8;
        while i < n - (n % 8) {
            for j in 0..8 {
                r[j] += a[i + j];
            }
            i += 8;
        }
        let mut res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
        while i < n {
            res += a[i];
            i += 1;
        }
        -0.0 + res
    };
    sum / n as f64
}

pub struct Entry {
    pub index: usize,
    pub frame: Vec<f32>,
    pub level: f64,
}

/// `detect.detect_frames` fed one blurred frame at a time.
pub struct Detector<'a> {
    cfg: &'a DetectConfig,
    half: usize,
    width: usize,
    height: usize,
    scale: f64,
    parallel: bool,
    mask: Vec<bool>,
    window: VecDeque<Entry>,
    residual: Vec<f64>,
    thresholds: Vec<f64>,
}

impl<'a> Detector<'a> {
    pub fn new(cfg: &'a DetectConfig, fps: f64, width: usize, height: usize, scale: f64, parallel: bool) -> Self {
        let n = width * height;
        Self {
            cfg,
            half: cfg.half_window(fps),
            width,
            height,
            scale,
            parallel,
            mask: vec![true; n],
            window: VecDeque::new(),
            residual: vec![0.0; n],
            thresholds: vec![0.0; n],
        }
    }

    pub fn push(&mut self, entry: Entry) -> Option<(usize, Vec<Blob>)> {
        let index = entry.index;
        if self.window.len() == 2 * self.half + 1 {
            self.window.pop_front();
        }
        self.window.push_back(entry);
        (index >= self.half).then(|| self.process(index - self.half))
    }

    pub fn finish(&mut self, last: usize) -> Vec<(usize, Vec<Blob>)> {
        let start = (last + 1).saturating_sub(self.half).max(0);
        (start..=last).map(|t| self.process(t)).collect()
    }

    fn process(&mut self, target: usize) -> (usize, Vec<Blob>) {
        let entries: Vec<&Entry> = self.window.iter().filter(|e| e.index.abs_diff(target) <= self.half).collect();
        let current = entries.iter().find(|e| e.index == target).expect("target in window");
        let sampled: Vec<&Entry> = entries.iter().step_by(self.cfg.bg_step).copied().collect();
        let levels: Vec<f64> = sampled.iter().map(|e| e.level).collect();
        let mean_level = mean_f64(&levels);
        let gains: Vec<f64> = levels.iter().map(|l| l - mean_level).collect();
        let use_noise = self.cfg.noise_factor > 0.0 && sampled.len() >= MIN_NOISE_SAMPLES;
        let frames: Vec<&[f32]> = sampled.iter().map(|e| e.frame.as_slice()).collect();
        let params = FrameParams {
            gain_offset: current.level - mean_level,
            gains: &gains,
            threshold: self.cfg.threshold,
            noise_factor: self.cfg.noise_factor,
            use_noise,
        };
        let any = residual_and_threshold(
            &frames,
            &current.frame,
            &self.mask,
            &params,
            &mut self.residual,
            &mut self.thresholds,
            self.parallel,
        );
        if !any {
            self.thresholds.iter_mut().for_each(|t| *t = self.cfg.threshold);
        }
        let blob_cfg =
            BlobConfig { min_area: self.cfg.min_area, max_area: self.cfg.max_area, merge_radius: self.cfg.merge_radius };
        (target, find_blobs_sparse(&self.residual, &self.thresholds, self.width, self.height, &blob_cfg, self.scale))
    }
}
