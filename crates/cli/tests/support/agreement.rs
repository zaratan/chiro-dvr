use std::path::Path;

use batdetect::jobs::Job;
use batdetect::process::{Processed, Versions};
use reference::recording::Recording;
use reference::track_matching::{Discrepancy, match_tracks};
use types::rate::FrameRate;

pub(crate) const VERSIONS: Versions<'static> = Versions {
    program: "test",
    ffmpeg: "test",
};

#[derive(Debug)]
pub(crate) struct Agreement {
    pub(crate) processed: Processed,
    pub(crate) reference: usize,
    pub(crate) paired: usize,
    pub(crate) agreeing: usize,
    pub(crate) lost: usize,
    pub(crate) added: usize,
    pub(crate) discrepancies: Vec<Discrepancy>,
}

pub(crate) fn job(video: &Path, dest: &Path) -> Job {
    Job {
        video: video.to_path_buf(),
        dest: dest.to_path_buf(),
    }
}

pub(crate) fn agreement(processed: Processed, recording: &Recording, rate: FrameRate) -> Agreement {
    let matching = match_tracks(&recording.tracks, &processed.tracks);
    let (paired, agreeing) = (matching.pairs.len(), matching.agreeing());
    let (lost, added) = (matching.reference_only.len(), matching.candidate_only.len());
    let discrepancies = matching.discrepancies(&recording.summary.video.name, rate);
    Agreement {
        reference: recording.tracks.len(),
        paired,
        agreeing,
        lost,
        added,
        discrepancies,
        processed,
    }
}
