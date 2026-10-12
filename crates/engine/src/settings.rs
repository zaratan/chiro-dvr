use std::fmt;

use serde::Deserialize;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Deserialize)]
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
        Self {
            threshold: 12.0,
            min_area: 4.0,
            max_area: 2700.0,
            bg_window_s: 1.0,
            bg_step: 3,
            osd_regions: Vec::new(),
            merge_radius: 6.0,
            work_width: 960,
            noise_factor: 8.0,
            target_sigma: 1.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Region {
    pub(crate) fn is_valid(self) -> bool {
        (0.0 <= self.x0 && self.x0 < self.x1 && self.x1 <= 1.0)
            && (0.0 <= self.y0 && self.y0 < self.y1 && self.y1 <= 1.0)
    }
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
}

impl fmt::Display for Bound {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Positive => formatter.write_str("a finite number > 0"),
            Self::NotNegative => formatter.write_str("a finite number ≥ 0"),
            Self::AtLeast(minimum) => write!(formatter, "an integer ≥ {minimum}"),
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
    #[error("the engine cannot handle these settings yet")]
    Unsupported(#[from] Unsupported),
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Unsupported {
    #[error("osd_regions: masks are not handled, {count} given")]
    OsdRegions { count: usize },
}
