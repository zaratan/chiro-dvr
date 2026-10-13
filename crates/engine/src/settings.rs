use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::config::DetectConfig;
use crate::convert::{position, real};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DetectSettings {
    pub threshold: f64,
    pub min_area: f64,
    pub max_area: f64,
    pub bg_window_s: f64,
    pub bg_step: usize,
    pub osd_regions: Vec<Region>,
    pub merge_radius: f64,
    pub work_width: u32,
    pub noise_factor: f64,
    pub target_sigma: f64,
}

impl Default for DetectSettings {
    fn default() -> Self {
        Self::from(&DetectConfig::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("expected four fractions X0,Y0,X1,Y1")]
pub struct RegionShape;

impl FromStr for Region {
    type Err = RegionShape;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bounds: Vec<f64> = text
            .split(',')
            .map(|bound| bound.trim().parse().ok())
            .collect::<Option<_>>()
            .ok_or(RegionShape)?;
        match bounds[..] {
            [x0, y0, x1, y1] => Ok(Self { x0, y0, x1, y1 }),
            _ => Err(RegionShape),
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{},{},{},{}", self.x0, self.y0, self.x1, self.y1)
    }
}

impl Region {
    pub fn columns(self, width: usize, margin: usize) -> Range<usize> {
        span(self.x0, self.x1, width, margin)
    }

    pub fn rows(self, height: usize, margin: usize) -> Range<usize> {
        span(self.y0, self.y1, height, margin)
    }

    pub fn contains(self, x: f64, y: f64) -> bool {
        (self.x0..=self.x1).contains(&x) && (self.y0..=self.y1).contains(&y)
    }

    pub(crate) fn is_valid(self) -> bool {
        (0.0 <= self.x0 && self.x0 < self.x1 && self.x1 <= 1.0)
            && (0.0 <= self.y0 && self.y0 < self.y1 && self.y1 <= 1.0)
    }
}

fn span(start: f64, end: f64, length: usize, margin: usize) -> Range<usize> {
    let last = position((end * real(length)).ceil()).saturating_add(margin).min(length);
    let first = position((start * real(length)).floor()).saturating_sub(margin);
    first.min(last)..last
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    Threshold,
    MinArea,
    MaxArea,
    BgWindowS,
    BgStep,
    MergeRadius,
    WorkWidth,
    NoiseFactor,
    TargetSigma,
}

impl Setting {
    pub fn flag(self) -> &'static str {
        match self {
            Self::Threshold => "threshold",
            Self::MinArea => "min-area",
            Self::MaxArea => "max-area",
            Self::BgWindowS => "bg-window",
            Self::BgStep => "bg-step",
            Self::MergeRadius => "merge-radius",
            Self::WorkWidth => "work-width",
            Self::NoiseFactor => "noise-factor",
            Self::TargetSigma => "target-sigma",
        }
    }
}

impl fmt::Display for Setting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Threshold => "threshold",
            Self::MinArea => "min_area",
            Self::MaxArea => "max_area",
            Self::BgWindowS => "bg_window_s",
            Self::BgStep => "bg_step",
            Self::MergeRadius => "merge_radius",
            Self::WorkWidth => "work_width",
            Self::NoiseFactor => "noise_factor",
            Self::TargetSigma => "target_sigma",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    Positive,
    NotNegative,
    AtLeast(u32),
    UpTo(u32),
}

impl fmt::Display for Bound {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Positive => formatter.write_str("a finite number > 0"),
            Self::NotNegative => formatter.write_str("a finite number ≥ 0"),
            Self::AtLeast(minimum) => write!(formatter, "an integer ≥ {minimum}"),
            Self::UpTo(most) => write!(formatter, "a finite number between 0 and {most}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{setting} must be {bound}, not {value}")]
    Setting { setting: Setting, value: f64, bound: Bound },
    #[error("min_area {min_area} is larger than max_area {max_area}")]
    Areas { min_area: f64, max_area: f64 },
    #[error("OSD region {index} does not satisfy 0 ≤ x0 < x1 ≤ 1 and 0 ≤ y0 < y1 ≤ 1: {region:?}")]
    Region { index: usize, region: Region },
}
