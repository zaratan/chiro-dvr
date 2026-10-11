use types::{detection, span, track};

use crate::track::TrackSetting;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("pad_s must be a finite number >= 0, not {0}")]
    Padding(f64),
    #[error("{setting} is out of bounds: {value}")]
    Setting { setting: TrackSetting, value: f64 },
    #[error("the kept detections cannot be rebuilt")]
    Detections(#[from] detection::Error),
    #[error("an unstable span cannot be built")]
    Span(#[from] span::Error),
    #[error("a track cannot be built")]
    Track(#[from] track::Error),
}
