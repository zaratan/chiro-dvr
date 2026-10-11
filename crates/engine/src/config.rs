use std::num::{NonZeroU32, NonZeroUsize};

use types::rate::FrameRate;

#[cfg(test)]
mod tests;

pub(crate) const MIN_BACKGROUND_FRAMES: usize = 3;
pub(crate) const MAX_BACKGROUND_SAMPLES: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub struct DetectConfig {
    threshold: f64,
    min_area: f64,
    max_area: f64,
    window_seconds: f64,
    step: NonZeroUsize,
    merge_radius: f64,
    work_width: NonZeroU32,
    noise_factor: f64,
    target_sigma: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(
        "a background window of {seconds} s covers {frames} frames at {rate} frames per second, fewer than \
         {MIN_BACKGROUND_FRAMES}"
    )]
    ShortWindow { seconds: f64, rate: f64, frames: usize },
    #[error("one frame in {step} keeps {samples} of {frames} window frames, fewer than {MIN_BACKGROUND_FRAMES}")]
    FewSamples { step: usize, frames: usize, samples: usize },
    #[error("one frame in {step} keeps {samples} of {frames} window frames, more than {MAX_BACKGROUND_SAMPLES}")]
    ManySamples { step: usize, frames: usize, samples: usize },
}

impl Default for DetectConfig {
    fn default() -> Self {
        Self {
            threshold: 12.0,
            min_area: 4.0,
            max_area: 2700.0,
            window_seconds: 1.0,
            step: NonZeroUsize::MIN.saturating_add(2),
            merge_radius: 6.0,
            work_width: NonZeroU32::MIN.saturating_add(959),
            noise_factor: 8.0,
            target_sigma: 1.5,
        }
    }
}

impl DetectConfig {
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    pub fn min_area(&self) -> f64 {
        self.min_area
    }

    pub fn max_area(&self) -> f64 {
        self.max_area
    }

    pub fn window_seconds(&self) -> f64 {
        self.window_seconds
    }

    pub fn step(&self) -> NonZeroUsize {
        self.step
    }

    pub fn merge_radius(&self) -> f64 {
        self.merge_radius
    }

    pub fn work_width(&self) -> NonZeroU32 {
        self.work_width
    }

    pub fn noise_factor(&self) -> f64 {
        self.noise_factor
    }

    pub fn target_sigma(&self) -> f64 {
        self.target_sigma
    }

    pub fn half_window(&self, rate: FrameRate) -> Result<NonZeroUsize, Error> {
        let rate = rate.get().to_f64();
        let half = (self.window_seconds * rate / 2.0).round() as usize;
        let frames = 2 * half + 1;
        let Some(nonzero) = NonZeroUsize::new(half) else {
            return Err(Error::ShortWindow {
                seconds: self.window_seconds,
                rate,
                frames,
            });
        };
        let step = self.step.get();
        let samples = frames.div_ceil(step);
        if samples < MIN_BACKGROUND_FRAMES {
            Err(Error::FewSamples { step, frames, samples })
        } else if samples > MAX_BACKGROUND_SAMPLES {
            Err(Error::ManySamples { step, frames, samples })
        } else {
            Ok(nonzero)
        }
    }
}
