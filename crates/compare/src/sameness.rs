use std::io::{self, Write};

use reference::frames::{FrameEntry, Frames};
use types::frame::FrameIndex;
use types::span::Span;

#[cfg(test)]
mod tests;

pub const LISTED_FRAMES: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sameness {
    name: String,
    verdict: Verdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Compared {
        outside: Vec<FrameIndex>,
        differing_inside: usize,
        span_count: usize,
        damaged_frames: usize,
    },
    InputsDiffer(InputGap),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputGap {
    FrameCount { previous: usize, current: usize },
    Entry { frame: FrameIndex, fields: Vec<Field> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Pts,
    KeyFrame,
    DecodeError,
}

impl Sameness {
    pub fn of(name: &str, previous: &Frames, current: &Frames, damaged: &[Span]) -> Self {
        Self {
            name: name.to_owned(),
            verdict: input_gap(previous.entries(), current.entries()).map_or_else(
                || detections_compared(previous, current, damaged),
                Verdict::InputsDiffer,
            ),
        }
    }

    pub fn verdict(&self) -> &Verdict {
        &self.verdict
    }

    pub fn holds(&self) -> bool {
        matches!(&self.verdict, Verdict::Compared { outside, .. } if outside.is_empty())
    }

    pub fn write_line(&self, out: &mut impl Write) -> io::Result<()> {
        match &self.verdict {
            Verdict::Compared {
                outside,
                differing_inside,
                span_count,
                damaged_frames,
            } => {
                if outside.is_empty() {
                    write!(out, "{}\tidentical", self.name)?;
                } else {
                    write!(
                        out,
                        "{}\tdiffers\toutside {}: {}",
                        self.name,
                        outside.len(),
                        listed(outside)
                    )?;
                }
                writeln!(
                    out,
                    "\tspans {span_count} ({damaged_frames} frames)\tdiffering inside {differing_inside}"
                )
            }
            Verdict::InputsDiffer(InputGap::FrameCount { previous, current }) => {
                writeln!(
                    out,
                    "{}\tinputs differ\t{previous} frames before, {current} now",
                    self.name
                )
            }
            Verdict::InputsDiffer(InputGap::Entry { frame, fields }) => {
                let fields: Vec<&str> = fields.iter().map(|field| field.label()).collect();
                writeln!(
                    out,
                    "{}\tinputs differ\tframe {frame}: {}",
                    self.name,
                    fields.join(", ")
                )
            }
        }
    }
}

impl Field {
    fn label(self) -> &'static str {
        match self {
            Self::Pts => "pts",
            Self::KeyFrame => "key frame",
            Self::DecodeError => "decode error",
        }
    }
}

fn input_gap(previous: &[FrameEntry], current: &[FrameEntry]) -> Option<InputGap> {
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
        .find(|(_, (before, now))| before != now)
        .map(|(frame, (before, now))| InputGap::Entry {
            frame: FrameIndex::new(frame),
            fields: [
                (before.pts != now.pts).then_some(Field::Pts),
                (before.key_frame != now.key_frame).then_some(Field::KeyFrame),
                (before.decode_error != now.decode_error).then_some(Field::DecodeError),
            ]
            .into_iter()
            .flatten()
            .collect(),
        })
}

fn detections_compared(previous: &Frames, current: &Frames, damaged: &[Span]) -> Verdict {
    let (inside, outside): (Vec<FrameIndex>, Vec<FrameIndex>) = previous
        .detections()
        .iter()
        .zip(current.detections().iter())
        .filter(|((_, before), (_, now))| before != now)
        .map(|((frame, _), _)| frame)
        .partition(|&frame| damaged.iter().any(|span| span.contains(frame)));
    Verdict::Compared {
        outside,
        differing_inside: inside.len(),
        span_count: damaged.len(),
        damaged_frames: damaged.iter().map(|span| span.frame_count()).sum(),
    }
}

fn listed(frames: &[FrameIndex]) -> String {
    let shown: Vec<String> = frames.iter().take(LISTED_FRAMES).map(ToString::to_string).collect();
    let rest = frames.len().saturating_sub(LISTED_FRAMES);
    if rest == 0 {
        shown.join(", ")
    } else {
        format!("{} … and {rest} more", shown.join(", "))
    }
}
