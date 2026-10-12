use types::{detection, span, track};

use crate::setting::Setting;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{setting} is out of bounds: {value}")]
    Setting { setting: Setting, value: f64 },
    #[error("the kept detections cannot be rebuilt")]
    Detections(#[from] detection::Error),
    #[error("an unstable span cannot be built")]
    Span(#[from] span::Error),
    #[error("a track cannot be built")]
    Track(#[from] track::Error),
}
