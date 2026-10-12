use std::io::{self, Write};

use reference::detection_matching::match_detections;
use reference::recording::Recording;
use types::detection::{Detection, FrameDetections};
use types::rate::FrameRate;
use types::span::Span;
use types::track::Track;

#[cfg(test)]
mod tests;

pub const MIN_RATE: f64 = 0.98;
const GROUP_GAP_FRAMES: usize = 30;
const SAME_POINT_PX: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq)]
pub struct Agreement {
    name: String,
    rate: FrameRate,
    reference: usize,
    candidate: usize,
    paired: usize,
    gaps: Vec<(Detection, Gap)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gap {
    Lost { on_track: bool },
    Added,
}

impl Agreement {
    pub fn of(name: &str, detections: &FrameDetections, rate: FrameRate, recording: &Recording) -> Self {
        let ignored = ignored_spans(recording);
        let reference = recording.frames.detections().without(&ignored);
        let candidate = detections.without(&ignored);
        let matching = match_detections(&reference, &candidate);
        let on_track = |lost: &Detection| {
            recording
                .tracks
                .iter()
                .flat_map(Track::points)
                .any(|point| point.frame == lost.frame && point.distance(lost) < SAME_POINT_PX)
        };
        let mut gaps: Vec<(Detection, Gap)> = matching
            .reference_only
            .iter()
            .map(|lost| {
                (
                    *lost,
                    Gap::Lost {
                        on_track: on_track(lost),
                    },
                )
            })
            .chain(matching.candidate_only.iter().map(|added| (*added, Gap::Added)))
            .collect();
        gaps.sort_by_key(|(detection, _)| detection.frame);
        Self {
            name: name.to_owned(),
            rate,
            reference: reference.detection_count(),
            candidate: candidate.detection_count(),
            paired: matching.pairs.len(),
            gaps,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn reference_rate(&self) -> f64 {
        rate(self.paired, self.reference)
    }

    pub fn candidate_rate(&self) -> f64 {
        rate(self.paired, self.candidate)
    }

    pub fn holds(&self) -> bool {
        self.reference_rate() >= MIN_RATE && self.candidate_rate() >= MIN_RATE
    }

    pub fn write_summary(&self, out: &mut impl Write) -> io::Result<()> {
        let (lost, on_tracks, added) = counts(&self.gaps);
        writeln!(
            out,
            "{}\treference {}\trust {}\tpaired {}\treference rate {:.4}\trust rate {:.4}\tlost {lost} ({on_tracks} on tracks)\tadded {added}",
            self.name,
            self.reference,
            self.candidate,
            self.paired,
            self.reference_rate(),
            self.candidate_rate(),
        )
    }

    pub fn write_groups(&self, out: &mut impl Write) -> io::Result<()> {
        let mut start = 0;
        for end in 1..=self.gaps.len() {
            let split = self
                .gaps
                .get(end)
                .is_none_or(|(next, _)| next.frame.get() > self.gaps[end - 1].0.frame.get() + GROUP_GAP_FRAMES);
            if split {
                self.write_group(out, &self.gaps[start..end])?;
                start = end;
            }
        }
        Ok(())
    }

    fn write_group(&self, out: &mut impl Write, group: &[(Detection, Gap)]) -> io::Result<()> {
        let (start, end) = (group[0].0.frame, group[group.len() - 1].0.frame);
        let (lost, on_tracks, added) = counts(group);
        let count = group.len() as f64;
        let (x, y) = group
            .iter()
            .fold((0.0, 0.0), |(x, y), (detection, _)| (x + detection.x, y + detection.y));
        let peak = group
            .iter()
            .map(|(detection, _)| detection.amplitude)
            .fold(0.0, f64::max);
        writeln!(
            out,
            "  {} {} frames {}-{}\tlost {lost} ({on_tracks} on tracks)\tadded {added}\tat ({:.0}, {:.0})\tpeak {peak:.1}",
            self.name,
            clock(self.rate.seconds(start)),
            start.get(),
            end.get(),
            x / count,
            y / count
        )
    }
}

pub(crate) fn clock(seconds: f64) -> String {
    let hundredths = (seconds * 100.0).round() as u64;
    format!(
        "{:02}:{:02}.{:02}",
        hundredths / 6000,
        hundredths / 100 % 60,
        hundredths % 100
    )
}

fn counts(gaps: &[(Detection, Gap)]) -> (usize, usize, usize) {
    let lost = gaps.iter().filter(|(_, gap)| matches!(gap, Gap::Lost { .. })).count();
    let on_tracks = gaps
        .iter()
        .filter(|(_, gap)| *gap == Gap::Lost { on_track: true })
        .count();
    (lost, on_tracks, gaps.len() - lost)
}

fn ignored_spans(recording: &Recording) -> Vec<Span> {
    let summary = &recording.summary;
    summary
        .damaged_spans
        .iter()
        .chain(&summary.unstable_spans)
        .copied()
        .collect()
}

fn rate(paired: usize, total: usize) -> f64 {
    if total == 0 { 1.0 } else { paired as f64 / total as f64 }
}
