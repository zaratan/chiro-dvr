pub mod blobs;
pub mod kernel;
pub mod network;
pub mod prep;

use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;

use rayon::prelude::*;

use blobs::{Blob, BlobConfig, find_blobs};
use kernel::{Above, Params, above_threshold};
use network::Network;
use prep::UNITS_PER_LEVEL;

pub const MIN_NOISE_SAMPLES: usize = 7;

pub struct Config {
    pub threshold: f64,
    pub min_area: f64,
    pub max_area: f64,
    pub bg_window_s: f64,
    pub bg_step: usize,
    pub merge_radius: f64,
    pub noise_factor: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self { threshold: 12.0, min_area: 4.0, max_area: 2700.0, bg_window_s: 1.0, bg_step: 3, merge_radius: 6.0, noise_factor: 8.0 }
    }
}

impl Config {
    pub fn half_window(&self, fps: f64) -> usize {
        (self.bg_window_s * fps / 2.0).round_ties_even() as usize
    }

    /// Thresholds are in gray levels.
    pub fn threshold_units(&self) -> f32 {
        (self.threshold * f64::from(UNITS_PER_LEVEL)) as f32
    }

    pub fn units_to_gray(&self, units: f32) -> f64 {
        f64::from(units) / f64::from(UNITS_PER_LEVEL)
    }
}

pub struct Work {
    pub index: usize,
    pub data: Vec<u16>,
    pub level: f64,
}

pub struct Detector {
    pub cfg: Config,
    pub half: usize,
    pub width: usize,
    pub height: usize,
    pub scale: f64,
    networks: Vec<Network>,
    window: VecDeque<Arc<Work>>,
    next_target: usize,
    targets: Range<usize>,
    last_frame: Option<usize>,
}

impl Detector {
    /// `targets`: frames to detect; `last_frame`: index of the video's last frame when known in advance.
    pub fn new(
        cfg: Config,
        fps: f64,
        size: (usize, usize),
        scale: f64,
        targets: Range<usize>,
        last_frame: Option<usize>,
    ) -> Self {
        let half = cfg.half_window(fps);
        let networks = (0..=kernel::MAX_SAMPLES).map(|n| Network::median(n.max(1))).collect();
        Self {
            cfg,
            half,
            width: size.0,
            height: size.1,
            scale,
            networks,
            window: VecDeque::new(),
            next_target: targets.start,
            targets,
            last_frame,
        }
    }

    pub fn first_needed(&self) -> usize {
        self.targets.start.saturating_sub(self.half)
    }

    pub fn last_needed(&self) -> usize {
        let wanted = self.targets.end.saturating_sub(1) + self.half;
        self.last_frame.map_or(wanted, |l| wanted.min(l))
    }

    fn frame(&self, index: usize) -> &Arc<Work> {
        &self.window[index - self.window[0].index]
    }

    fn process(&self, target: usize, last: usize) -> (usize, Vec<Blob>) {
        let first = target.saturating_sub(self.half);
        let stop = (target + self.half).min(last);
        let sampled: Vec<&Arc<Work>> = (first..=stop).step_by(self.cfg.bg_step).map(|i| self.frame(i)).collect();
        let n = sampled.len();
        let mean_level = sampled.iter().map(|w| w.level).sum::<f64>() / n as f64;
        let gains: Vec<f32> = sampled.iter().map(|w| (w.level - mean_level) as f32).collect();
        let current = self.frame(target);
        let params = Params {
            threshold: self.cfg.threshold_units(),
            noise_factor: self.cfg.noise_factor as f32,
            use_noise: self.cfg.noise_factor > 0.0 && n >= MIN_NOISE_SAMPLES,
            gain_offset: (current.level - mean_level) as f32,
            gains: &gains,
        };
        let samples: Vec<&[u16]> = sampled.iter().map(|w| w.data.as_slice()).collect();
        let mut above: Vec<Above> = Vec::new();
        above_threshold(&samples, &current.data, &self.networks[n], &params, &mut above);
        let blob_cfg =
            BlobConfig { min_area: self.cfg.min_area, max_area: self.cfg.max_area, merge_radius: self.cfg.merge_radius };
        (target, find_blobs(&above, self.width, self.height, &blob_cfg, self.scale))
    }

    /// Adds prepared frames (consecutive indices) and returns the targets whose window is complete.
    pub fn push(&mut self, frames: Vec<Work>, finished: bool) -> Vec<(usize, Vec<Blob>)> {
        for f in frames {
            self.window.push_back(Arc::new(f));
        }
        let Some(last) = self.window.back().map(|w| w.index) else { return Vec::new() };
        if finished {
            self.last_frame = Some(last);
        }
        let at_end = self.last_frame == Some(last);
        let ready = if at_end { last + 1 } else { (last + 1).saturating_sub(self.half) };
        let ready_until = ready.min(self.targets.end);
        let targets: Vec<usize> = (self.next_target..ready_until).collect();
        let found: Vec<(usize, Vec<Blob>)> = targets.par_iter().map(|&t| self.process(t, last)).collect();
        self.next_target = self.next_target.max(ready_until);
        let keep_from = self.next_target.saturating_sub(self.half);
        while self.window.front().is_some_and(|w| w.index < keep_from) {
            self.window.pop_front();
        }
        found
    }
}
