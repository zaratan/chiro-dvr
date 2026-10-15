use std::path::{Path, PathBuf};

use batdetect::options::Options;
use batdetect::process::{self, Tracked};
use engine::detection::{Detected, detect};
use files::time::frame_seconds;
use media::decode::Threads;
use media::fingerprint;
use render::clips::{Clip, Outputs};
use render::drawn::drawn;
use render::source::Source;
use types::timeline::Timeline;
use types::track::Track;

const HALF_HUNDREDTH_S: f64 = 0.005;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Frozen {
    pub(crate) frames: usize,
    pub(crate) fingerprint: u64,
}

pub(crate) struct Drawing {
    video: PathBuf,
    detected: Detected,
    timeline: Timeline,
    tracks: Vec<Track>,
    options: Options,
}

pub(crate) fn clip(wanted: Vec<Clip>) -> Outputs {
    Outputs {
        clips: wanted,
        zooms: Vec::new(),
        annotated: None,
    }
}

#[cfg(test)]
pub(crate) fn tracked(video: &Path) -> Drawing {
    let options = Options::default();
    let detected = detect(video, Threads::Automatic, &options.detect).unwrap();
    let Tracked { timeline, tracks, .. } = process::tracked(&detected, &options).unwrap();
    Drawing {
        video: video.to_path_buf(),
        detected,
        timeline,
        tracks,
        options,
    }
}

#[cfg(test)]
impl Drawing {
    pub(crate) fn frozen(&self, start_s: f64, view: fn(Vec<Clip>) -> Outputs) -> Frozen {
        let track = self
            .tracks
            .iter()
            .position(|track| (frame_seconds(&self.timeline, track.first().frame) - start_s).abs() < HALF_HUNDREDTH_S)
            .unwrap_or_else(|| panic!("no track starts at {start_s} s"));
        let reading = self.detected.reading();
        let errors = self.detected.damage().error_frames();
        let source = Source::new(&self.video, &self.timeline, &reading.key_frames(), errors).unwrap();
        let wanted = vec![Clip {
            track,
            path: PathBuf::from("frozen.mp4"),
        }];
        let found = drawn(&source, &self.tracks, &view(wanted), self.options.render).unwrap();
        let fingerprints = &found[0].fingerprints;
        Frozen {
            frames: fingerprints.len(),
            fingerprint: fingerprint::of_sequence(fingerprints.iter().copied()),
        }
    }
}
