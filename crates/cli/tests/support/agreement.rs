use std::path::Path;

use batdetect::jobs::Job;
use batdetect::process::{Processed, Versions, Wording};
use reference::recording::Recording;
use reference::track_matching::{Discrepancy, match_tracks};
use summary::header::{Band, HeaderLine, Labels};
use types::language::Language;
use types::rate::FrameRate;

pub(crate) const VERSIONS: Versions<'static> = Versions {
    program: "test",
    ffmpeg: "test",
};

#[derive(Debug)]
pub(crate) struct DebugLabels;

impl Labels for DebugLabels {
    fn line(&self, line: &HeaderLine) -> String {
        format!("{line:?}")
    }

    fn band(&self, band: Band) -> String {
        format!("{band:?}")
    }
}

impl Wording for DebugLabels {
    fn language(&self) -> Language {
        Language::English
    }
}

#[derive(Debug)]
pub(crate) struct Agreement {
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

pub(crate) fn agreement(processed: &Processed, recording: &Recording, rate: FrameRate) -> Agreement {
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
    }
}
