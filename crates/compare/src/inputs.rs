use std::fmt;

use reference::frames::FrameEntry;
use types::frame::FrameIndex;
use types::losses::EdgeLosses;

#[cfg(test)]
mod tests;

pub const STRICT_FIELDS: [Field; 3] = [Field::Pts, Field::KeyFrame, Field::DecodeError];
pub const REFERENCE_FIELDS: [Field; 2] = [Field::Pts, Field::KeyFrame];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputGap {
    FrameCount { previous: usize, current: usize },
    Entry { frame: FrameIndex, fields: Vec<Field> },
    Losses { previous: EdgeLosses, current: EdgeLosses },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Pts,
    KeyFrame,
    DecodeError,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DecodeErrors {
    pub reference: usize,
    pub current: usize,
    pub differing: usize,
}

pub fn entry_gap(previous: &[FrameEntry], current: &[FrameEntry], compared: &[Field]) -> Option<InputGap> {
    if previous.len() != current.len() {
        return Some(InputGap::FrameCount {
            previous: previous.len(),
            current: current.len(),
        });
    }
    previous
        .iter()
        .zip(current)
        .enumerate()
        .find_map(|(frame, (before, now))| {
            let fields: Vec<Field> = compared
                .iter()
                .copied()
                .filter(|field| field.differs(before, now))
                .collect();
            (!fields.is_empty()).then(|| InputGap::Entry {
                frame: FrameIndex::new(frame),
                fields,
            })
        })
}

pub fn losses_gap(previous: EdgeLosses, current: EdgeLosses) -> Option<InputGap> {
    (previous != current).then_some(InputGap::Losses { previous, current })
}

pub fn decode_errors(reference: &[FrameEntry], current: &[FrameEntry]) -> DecodeErrors {
    let in_error = |entries: &[FrameEntry]| entries.iter().filter(|entry| entry.decode_error).count();
    DecodeErrors {
        reference: in_error(reference),
        current: in_error(current),
        differing: reference
            .iter()
            .zip(current)
            .filter(|(before, now)| before.decode_error != now.decode_error)
            .count(),
    }
}

impl Field {
    fn differs(self, before: &FrameEntry, now: &FrameEntry) -> bool {
        match self {
            Self::Pts => before.pts != now.pts,
            Self::KeyFrame => before.key_frame != now.key_frame,
            Self::DecodeError => before.decode_error != now.decode_error,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Pts => "pts",
            Self::KeyFrame => "key frame",
            Self::DecodeError => "decode error",
        }
    }
}

impl fmt::Display for InputGap {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameCount { previous, current } => write!(out, "{previous} frames before, {current} now"),
            Self::Entry { frame, fields } => {
                let fields: Vec<&str> = fields.iter().map(|field| field.label()).collect();
                write!(out, "frame {frame}: {}", fields.join(", "))
            }
            Self::Losses { previous, current } => write!(
                out,
                "frames lost at the start and at the end: {}/{} before, {}/{} now",
                previous.before_first, previous.after_last, current.before_first, current.after_last
            ),
        }
    }
}
