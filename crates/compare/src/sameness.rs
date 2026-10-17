use std::io::{self, Write};

use reference::frames::Frames;
use types::frame::FrameIndex;
use types::span::Span;

use crate::inputs::{InputGap, STRICT_FIELDS, entry_gap};

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

impl Sameness {
    pub fn of(name: &str, previous: &Frames, current: &Frames, damaged: &[Span]) -> Self {
        Self {
            name: name.to_owned(),
            verdict: entry_gap(previous.entries(), current.entries(), &STRICT_FIELDS).map_or_else(
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
            Verdict::InputsDiffer(gap) => writeln!(out, "{}\tinputs differ\t{gap}", self.name),
        }
    }
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

pub(crate) fn listed(frames: &[FrameIndex]) -> String {
    let shown: Vec<String> = frames.iter().take(LISTED_FRAMES).map(ToString::to_string).collect();
    let rest = frames.len().saturating_sub(LISTED_FRAMES);
    if rest == 0 {
        shown.join(", ")
    } else {
        format!("{} … and {rest} more", shown.join(", "))
    }
}
