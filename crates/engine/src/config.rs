use std::num::{NonZeroU32, NonZeroUsize};

use types::rate::FrameRate;

use crate::settings::{self, Bound, DetectSettings, Region, Setting};
use crate::window::{Unit, most_sampled};

#[cfg(test)]
mod tests;

pub(crate) const MIN_BACKGROUND_FRAMES: usize = 3;
pub(crate) const BACKGROUND_BUDGET: u64 = 4 << 30;
const BYTES_PER_WINDOW_PIXEL: f64 = size_of::<Unit>() as f64;
pub const MIN_WORK_WIDTH: u32 = 16;
pub(crate) const MAX_MERGE_RADIUS: u32 = 12;

#[derive(Debug, Clone, PartialEq)]
pub struct DetectConfig {
    threshold: f64,
    min_area: f64,
    max_area: f64,
    window_seconds: f64,
    step: NonZeroUsize,
    osd_regions: Vec<Region>,
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
    #[error(
        "a background window of {seconds} s at {rate} frames per second keeps {frames} frames of {pixels} \
         working pixels, more than the {BACKGROUND_BUDGET} bytes allowed"
    )]
    LongWindow {
        seconds: f64,
        rate: f64,
        frames: f64,
        pixels: usize,
    },
}

impl Default for DetectConfig {
    fn default() -> Self {
        Self {
            threshold: 12.0,
            min_area: 4.0,
            max_area: 2700.0,
            window_seconds: 1.0,
            step: NonZeroUsize::MIN.saturating_add(2),
            osd_regions: Vec::new(),
            merge_radius: 6.0,
            work_width: NonZeroU32::MIN.saturating_add(959),
            noise_factor: 8.0,
            target_sigma: 1.5,
        }
    }
}

impl TryFrom<&DetectSettings> for DetectConfig {
    type Error = settings::Error;

    fn try_from(settings: &DetectSettings) -> Result<Self, Self::Error> {
        positive(settings.threshold, Setting::Threshold)?;
        positive(settings.min_area, Setting::MinArea)?;
        positive(settings.max_area, Setting::MaxArea)?;
        if settings.min_area > settings.max_area {
            return Err(settings::Error::Areas {
                min_area: settings.min_area,
                max_area: settings.max_area,
            });
        }
        positive(settings.bg_window_s, Setting::BgWindowS)?;
        within(settings.merge_radius, MAX_MERGE_RADIUS, Setting::MergeRadius)?;
        not_negative(settings.noise_factor, Setting::NoiseFactor)?;
        not_negative(settings.target_sigma, Setting::TargetSigma)?;
        let step = NonZeroUsize::new(settings.bg_step).ok_or(settings::Error::Setting {
            setting: Setting::BgStep,
            value: 0.0,
            bound: Bound::AtLeast(1),
        })?;
        let work_width = NonZeroU32::new(settings.work_width)
            .filter(|width| width.get() >= MIN_WORK_WIDTH)
            .ok_or(settings::Error::Setting {
                setting: Setting::WorkWidth,
                value: f64::from(settings.work_width),
                bound: Bound::AtLeast(MIN_WORK_WIDTH),
            })?;
        if let Some((index, &region)) = settings
            .osd_regions
            .iter()
            .enumerate()
            .find(|(_, region)| !region.is_valid())
        {
            return Err(settings::Error::Region { index, region });
        }
        Ok(Self {
            threshold: settings.threshold,
            min_area: settings.min_area,
            max_area: settings.max_area,
            window_seconds: settings.bg_window_s,
            step,
            osd_regions: settings.osd_regions.clone(),
            merge_radius: settings.merge_radius,
            work_width,
            noise_factor: settings.noise_factor,
            target_sigma: settings.target_sigma,
        })
    }
}

fn positive(value: f64, setting: Setting) -> Result<(), settings::Error> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(settings::Error::Setting {
            setting,
            value,
            bound: Bound::Positive,
        })
    }
}

fn not_negative(value: f64, setting: Setting) -> Result<(), settings::Error> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(settings::Error::Setting {
            setting,
            value,
            bound: Bound::NotNegative,
        })
    }
}

fn within(value: f64, most: u32, setting: Setting) -> Result<(), settings::Error> {
    if (0.0..=f64::from(most)).contains(&value) {
        Ok(())
    } else {
        Err(settings::Error::Setting {
            setting,
            value,
            bound: Bound::UpTo(most),
        })
    }
}

impl From<&DetectConfig> for DetectSettings {
    fn from(config: &DetectConfig) -> Self {
        Self {
            threshold: config.threshold,
            min_area: config.min_area,
            max_area: config.max_area,
            bg_window_s: config.window_seconds,
            bg_step: config.step.get(),
            osd_regions: config.osd_regions.clone(),
            merge_radius: config.merge_radius,
            work_width: config.work_width.get(),
            noise_factor: config.noise_factor,
            target_sigma: config.target_sigma,
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

    pub fn osd_regions(&self) -> &[Region] {
        &self.osd_regions
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

    pub fn half_window(&self, rate: FrameRate, pixels: usize) -> Result<NonZeroUsize, Error> {
        let rate = rate.get().to_f64();
        let exact = (self.window_seconds * rate / 2.0).round();
        let window_frames = 2.0 * exact + 1.0;
        let fits =
            exact.is_finite() && window_frames * pixels as f64 * BYTES_PER_WINDOW_PIXEL <= BACKGROUND_BUDGET as f64;
        if !fits {
            return Err(Error::LongWindow {
                seconds: self.window_seconds,
                rate,
                frames: window_frames,
                pixels,
            });
        }
        let half = exact as usize;
        let frames = 2 * half + 1;
        let Some(nonzero) = NonZeroUsize::new(half) else {
            return Err(Error::ShortWindow {
                seconds: self.window_seconds,
                rate,
                frames,
            });
        };
        let step = self.step.get();
        let samples = most_sampled(nonzero, self.step).get();
        if samples < MIN_BACKGROUND_FRAMES {
            Err(Error::FewSamples { step, frames, samples })
        } else {
            Ok(nonzero)
        }
    }
}
