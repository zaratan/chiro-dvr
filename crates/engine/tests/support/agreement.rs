use std::io::{self, Write};

use engine::detection::Detected;
use reference::detection_matching::match_detections;
use reference::recording::Recording;
use types::detection::{Detection, FrameDetections};
use types::rate::FrameRate;
use types::span::Span;

pub(crate) const MIN_RATE: f64 = 0.98;
const GROUP_GAP_FRAMES: usize = 30;

pub(crate) struct Agreement {
    pub(crate) name: String,
    rate: FrameRate,
    reference: usize,
    rust: usize,
    paired: usize,
    lost: Vec<(Detection, bool)>,
    added: Vec<Detection>,
}

impl Agreement {
    pub(crate) fn of(detected: &Detected, recording: &Recording) -> Self {
        let summary = &recording.summary;
        let ignored: Vec<Span> = summary
            .damaged_spans
            .iter()
            .chain(&summary.unstable_spans)
            .copied()
            .collect();
        let reference = kept(recording.frames.detections(), &ignored);
        let rust = kept(detected.detections(), &ignored);
        let matching = match_detections(&reference, &rust);
        let on_track = |lost: &Detection| {
            recording
                .tracks
                .iter()
                .flat_map(types::track::Track::points)
                .any(|point| point.frame == lost.frame && point.distance(lost) < 1e-6)
        };
        Self {
            name: summary.video.name.clone(),
            rate: detected.reading().video().frame_rate(),
            reference: reference.detection_count(),
            rust: rust.detection_count(),
            paired: matching.pairs.len(),
            lost: matching
                .reference_only
                .iter()
                .map(|lost| (*lost, on_track(lost)))
                .collect(),
            added: matching.candidate_only,
        }
    }

    pub(crate) fn reference_rate(&self) -> f64 {
        rate(self.paired, self.reference)
    }

    pub(crate) fn rust_rate(&self) -> f64 {
        rate(self.paired, self.rust)
    }

    pub(crate) fn write_summary(&self, out: &mut impl Write) -> io::Result<()> {
        let on_tracks = self.lost.iter().filter(|(_, on_track)| *on_track).count();
        writeln!(
            out,
            "{}\treference {}\trust {}\tpaired {}\treference rate {:.4}\trust rate {:.4}\tlost {} ({} on tracks)\tadded {}",
            self.name,
            self.reference,
            self.rust,
            self.paired,
            self.reference_rate(),
            self.rust_rate(),
            self.lost.len(),
            on_tracks,
            self.added.len()
        )
    }

    pub(crate) fn write_groups(&self, out: &mut impl Write) -> io::Result<()> {
        let mut gaps: Vec<(Detection, Option<bool>)> = self
            .lost
            .iter()
            .map(|&(lost, on_track)| (lost, Some(on_track)))
            .chain(self.added.iter().map(|&added| (added, None)))
            .collect();
        gaps.sort_by_key(|(detection, _)| detection.frame);
        let mut start = 0;
        for end in 1..=gaps.len() {
            let split = gaps
                .get(end)
                .is_none_or(|(next, _)| next.frame.get() > gaps[end - 1].0.frame.get() + GROUP_GAP_FRAMES);
            if split {
                self.write_group(out, &gaps[start..end])?;
                start = end;
            }
        }
        Ok(())
    }

    fn write_group(&self, out: &mut impl Write, group: &[(Detection, Option<bool>)]) -> io::Result<()> {
        let (first, last) = (group[0].0.frame, group[group.len() - 1].0.frame);
        let seconds = self.rate.seconds(first);
        let missing = group.iter().filter(|(_, kind)| kind.is_some()).count();
        let on_tracks = group.iter().filter(|(_, kind)| *kind == Some(true)).count();
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
            "  {} {:02}:{:05.2} frames {}-{}\tlost {missing} ({on_tracks} on tracks)\tadded {}\tat ({:.0}, {:.0})\tpeak {peak:.1}",
            self.name,
            (seconds / 60.0).floor(),
            seconds % 60.0,
            first.get(),
            last.get(),
            group.len() - missing,
            x / count,
            y / count
        )
    }
}

fn kept(detections: &FrameDetections, ignored: &[Span]) -> FrameDetections {
    let frames = detections
        .iter()
        .map(|(frame, found)| {
            if ignored.iter().any(|span| span.contains(frame)) {
                Vec::new()
            } else {
                found.to_vec()
            }
        })
        .collect();
    FrameDetections::new(frames).unwrap()
}

fn rate(paired: usize, total: usize) -> f64 {
    if total == 0 { 1.0 } else { paired as f64 / total as f64 }
}
