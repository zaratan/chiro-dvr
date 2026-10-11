use types::{detection, span};

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("pad_s must be a finite number >= 0, not {0}")]
    Padding(f64),
    #[error("the kept detections cannot be rebuilt")]
    Detections(#[from] detection::Error),
    #[error("an unstable span cannot be built")]
    Span(#[from] span::Error),
}
