use std::num::NonZeroUsize;

use rayon::prelude::*;
use types::detection::{FrameDetections, Measures};

use crate::blur::Blur;
use crate::closing::{Closing, Disk};
use crate::components::Components;
use crate::config::{DetectConfig, MAX_BACKGROUND_SAMPLES};
use crate::convert::{narrowed, wide};
use crate::geometry::Geometry;
use crate::network::Networks;
use crate::prepare::Prepare;
use crate::threshold::{Gains, Lit, Threshold};
use crate::window::{Prepared, Window};

#[cfg(test)]
mod tests;

pub(crate) const BATCH: NonZeroUsize = NonZeroUsize::MIN.saturating_add(31);

#[derive(Debug)]
pub(crate) struct Detector {
    pixels: u64,
    batch: NonZeroUsize,
    window: Window,
    networks: Networks,
    threshold: Threshold,
    workers: Vec<Worker>,
    pending: Vec<Vec<u8>>,
    spare: Vec<Vec<u8>>,
    slots: Vec<Prepared>,
    found: FrameDetections,
}

#[derive(Debug)]
struct Worker {
    prepare: Prepare,
    closing: Closing,
    components: Components,
    lit: Vec<Lit>,
}

struct Shared<'a> {
    pixels: u64,
    window: &'a Window,
    networks: &'a Networks,
    threshold: Threshold,
}

impl Detector {
    pub(crate) fn workers(&self) -> usize {
        self.workers.len()
    }

    pub(crate) fn new(
        geometry: Geometry,
        blur: &Blur,
        disk: &Disk,
        half: NonZeroUsize,
        config: &DetectConfig,
        batch: NonZeroUsize,
    ) -> Self {
        Self {
            pixels: geometry.pixels() as u64,
            batch,
            window: Window::new(half, config.step(), geometry.pixels()),
            networks: Networks::new(),
            threshold: Threshold::new(config),
            workers: (0..rayon::current_num_threads())
                .map(|_| Worker::new(geometry, blur, disk, config))
                .collect(),
            pending: Vec::new(),
            spare: Vec::new(),
            slots: Vec::new(),
            found: FrameDetections::default(),
        }
    }

    pub(crate) fn push(&mut self, luma: &[u8]) {
        let mut copy = self.spare.pop().unwrap_or_default();
        copy.clear();
        copy.extend_from_slice(luma);
        self.pending.push(copy);
        if self.pending.len() == self.batch.get() {
            self.flush(false);
        }
    }

    pub(crate) fn finish(mut self) -> FrameDetections {
        self.flush(true);
        self.found
    }

    fn flush(&mut self, end: bool) {
        self.prepare_pending();
        let pushed = self.window.pushed();
        let half = self.window.half();
        let until = if end { pushed } else { pushed.saturating_sub(half) };
        let from = self.found.frame_count();
        if from < until {
            self.detect_targets(from, until, pushed - 1);
            self.window.release_before(until.saturating_sub(half));
        }
    }

    fn prepare_pending(&mut self) {
        let Self {
            window,
            workers,
            pending,
            spare,
            slots,
            ..
        } = self;
        slots.extend((0..pending.len()).map(|_| window.slot()));
        let group = pending.len().div_ceil(workers.len()).max(1);
        workers
            .par_iter_mut()
            .zip(pending.par_chunks(group))
            .zip(slots.par_chunks_mut(group))
            .for_each(|((worker, lumas), slots)| {
                for (luma, slot) in lumas.iter().zip(slots) {
                    slot.sum = worker.prepare.run(luma, &mut slot.units);
                }
            });
        for slot in slots.drain(..) {
            window.push(slot);
        }
        spare.append(pending);
    }

    fn detect_targets(&mut self, from: usize, until: usize, last: usize) {
        let shared = Shared {
            pixels: self.pixels,
            window: &self.window,
            networks: &self.networks,
            threshold: self.threshold,
        };
        let group = (until - from).div_ceil(self.workers.len());
        let found: Vec<Vec<Vec<Measures>>> = self
            .workers
            .par_iter_mut()
            .enumerate()
            .map(|(index, worker)| {
                let start = (from + index * group).min(until);
                (start..(start + group).min(until))
                    .map(|target| worker.detect(&shared, target, last))
                    .collect()
            })
            .collect();
        for measures in found.into_iter().flatten() {
            self.found.push(measures);
        }
    }
}

impl Worker {
    fn new(geometry: Geometry, blur: &Blur, disk: &Disk, config: &DetectConfig) -> Self {
        Self {
            prepare: Prepare::new(geometry, blur),
            closing: Closing::new(geometry, disk),
            components: Components::new(geometry, config),
            lit: Vec::new(),
        }
    }

    fn detect(&mut self, shared: &Shared<'_>, target: usize, last: usize) -> Vec<Measures> {
        let mut samples: [&[u16]; MAX_BACKGROUND_SAMPLES] = [&[]; MAX_BACKGROUND_SAMPLES];
        let mut sums = [0_u64; MAX_BACKGROUND_SAMPLES];
        let mut count = 0;
        for ((sample, sum), frame) in samples
            .iter_mut()
            .zip(&mut sums)
            .zip(shared.window.sampled(target, last))
        {
            (*sample, *sum) = (&frame.units, frame.sum);
            count += 1;
        }
        let mut found = Vec::new();
        let Some(inputs) = NonZeroUsize::new(count) else {
            return found;
        };
        let (samples, sums) = (&samples[..count], &sums[..count]);
        let total: u64 = sums.iter().sum();
        let mut gains = [0.0_f32; MAX_BACKGROUND_SAMPLES];
        for (gain, &sum) in gains.iter_mut().zip(sums) {
            *gain = gain_of(sum, total, count, shared.pixels);
        }
        let current = shared.window.frame(target);
        let gains = Gains {
            current: gain_of(current.sum, total, count, shared.pixels),
            samples: &gains[..count],
        };
        shared.threshold.lit(
            samples,
            &current.units,
            shared.networks.get(inputs),
            gains,
            &mut self.lit,
        );
        let closed = self.closing.run(&self.lit);
        self.components.run(closed, &self.lit, &mut found);
        found
    }
}

fn gain_of(sum: u64, total: u64, count: usize, pixels: u64) -> f32 {
    let count = count as u64;
    let excess = i128::from(sum) * i128::from(count) - i128::from(total);
    narrowed(wide(excess) / wide(i128::from(count * pixels)))
}
