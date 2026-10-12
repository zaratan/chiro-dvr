use std::io::{self, Write};
use std::iter::Sum;

use reference::recording::Recording;
use reference::track_matching::{Discrepancy, DiscrepancyKind, match_tracks};
use tracking::error::Error;
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};
use types::detection::FrameDetections;
use types::rate::FrameRate;

use crate::agreement::clock;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct TrackAgreement {
    name: String,
    counts: TrackCounts,
    discrepancies: Vec<Discrepancy>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrackCounts {
    pub reference: usize,
    pub paired: usize,
    pub agreeing: usize,
    pub lost: usize,
    pub added: usize,
}

impl TrackAgreement {
    pub fn of(name: &str, detections: &FrameDetections, rate: FrameRate, recording: &Recording) -> Result<Self, Error> {
        let summary = &recording.summary;
        let exclusion = exclude(
            detections,
            summary.damaged_spans.clone(),
            summary.counts.damaged_frames_reported,
            rate,
            StabilityConfig::default(),
        );
        let tracks = track_detections(&exclusion.detections, TrackConfig::default())?;
        let matching = match_tracks(&recording.tracks, &tracks);
        Ok(Self {
            name: name.to_owned(),
            counts: TrackCounts {
                reference: recording.tracks.len(),
                paired: matching.pairs.len(),
                agreeing: matching.agreeing(),
                lost: matching.reference_only.len(),
                added: matching.candidate_only.len(),
            },
            discrepancies: matching.discrepancies(name, rate),
        })
    }

    pub fn counts(&self) -> TrackCounts {
        self.counts
    }

    pub fn write_summary(&self, out: &mut impl Write) -> io::Result<()> {
        let TrackCounts {
            reference,
            paired,
            agreeing,
            lost,
            added,
        } = self.counts;
        writeln!(
            out,
            "{}\ttracks {reference}\tpaired {paired} (agreeing {agreeing})\tlost {lost}\tadded {added}",
            self.name
        )
    }

    pub fn write_discrepancies(&self, out: &mut impl Write) -> io::Result<()> {
        for discrepancy in &self.discrepancies {
            let what = match discrepancy.kind {
                DiscrepancyKind::Missed { reference_id } => format!("lost reference track {reference_id}"),
                DiscrepancyKind::Extra { candidate_id } => format!("added track {candidate_id}"),
                DiscrepancyKind::Points {
                    reference_id,
                    candidate_id,
                    reference_points,
                    candidate_points,
                } => format!(
                    "reference track {reference_id} has {reference_points} points, track {candidate_id} has {candidate_points}"
                ),
            };
            writeln!(
                out,
                "  {} {} frame {}\t{what}",
                discrepancy.video,
                clock(discrepancy.seconds),
                discrepancy.frame.get()
            )?;
        }
        Ok(())
    }
}

impl Sum for TrackCounts {
    fn sum<I: Iterator<Item = Self>>(counts: I) -> Self {
        counts.fold(Self::default(), |total, counts| Self {
            reference: total.reference + counts.reference,
            paired: total.paired + counts.paired,
            agreeing: total.agreeing + counts.agreeing,
            lost: total.lost + counts.lost,
            added: total.added + counts.added,
        })
    }
}
