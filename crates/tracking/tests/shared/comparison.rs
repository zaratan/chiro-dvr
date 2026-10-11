use reference::recording::Recording;
use reference::track_matching::{Discrepancy, match_tracks};
use tracking::error::Error;
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};
use types::rate::FrameRate;
use types::track::Track;

#[derive(Debug)]
pub(crate) struct Comparison {
    pub(crate) video: String,
    pub(crate) reference: usize,
    pub(crate) tracked: Vec<Track>,
    pub(crate) pairs: usize,
    pub(crate) agreeing: usize,
    pub(crate) reference_only: usize,
    pub(crate) tracked_only: usize,
    pub(crate) identical: usize,
    pub(crate) largest_gap: f64,
    pub(crate) different_frames: usize,
    pub(crate) discrepancies: Vec<Discrepancy>,
}

pub(crate) fn compare(recording: &Recording, rate: FrameRate) -> Result<Comparison, Error> {
    let summary = &recording.summary;
    let exclusion = exclude(
        recording.frames.detections(),
        summary.damaged_spans.clone(),
        summary.counts.damaged_frames_reported,
        rate,
        StabilityConfig::default(),
    )?;
    let tracked = track_detections(&exclusion.detections, TrackConfig::default())?;
    let matching = match_tracks(&recording.tracks, &tracked);
    let gaps: Vec<Option<f64>> = matching.pairs.iter().map(|(kept, found)| gap(kept, found)).collect();
    let comparison = Comparison {
        video: summary.video.name.clone(),
        reference: recording.tracks.len(),
        pairs: matching.pairs.len(),
        agreeing: matching.agreeing(),
        reference_only: matching.reference_only.len(),
        tracked_only: matching.candidate_only.len(),
        identical: matching
            .pairs
            .iter()
            .filter(|(kept, found)| kept.id() == found.id() && kept.points() == found.points())
            .count(),
        largest_gap: gaps.iter().flatten().copied().fold(0.0, f64::max),
        different_frames: gaps.iter().filter(|gap| gap.is_none()).count(),
        discrepancies: matching.discrepancies(&summary.video.name, rate),
        tracked,
    };
    Ok(comparison)
}

fn gap(kept: &Track, found: &Track) -> Option<f64> {
    let (kept, found) = (kept.points(), found.points());
    if kept.len() != found.len() || kept.iter().zip(found).any(|(a, b)| a.frame != b.frame) {
        return None;
    }
    Some(
        kept.iter()
            .zip(found)
            .flat_map(|(a, b)| [(a.x - b.x).abs(), (a.y - b.y).abs(), (a.area - b.area).abs()])
            .fold(0.0, f64::max),
    )
}
