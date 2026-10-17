use std::path::{Path, PathBuf};

use draw::color::Rgb;

use batdetect::options::Options;
use batdetect::process::{self, Tracked};
use engine::detection::{Detected, detect};
use files::params::spans_in_seconds;
use files::time::frame_seconds;
use media::decode::Threads;
use media::fingerprint;
use render::clips::{Clip, Outputs};
use render::colored::Colored;
use render::drawn::drawn;
use render::source::Source;
use summary::assigned::track_colors;
use summary::layout::Summary;
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
    colors: Vec<Option<Rgb>>,
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
    let Tracked {
        timeline,
        exclusion,
        tracks,
    } = process::tracked(&detected, &options).unwrap();
    let name = video.file_stem().unwrap().to_string_lossy();
    let summary = Summary {
        name: &name,
        tracks: &tracks,
        timeline: &timeline,
        ignored: &spans_in_seconds(&exclusion.unstable, &timeline),
        damaged: &spans_in_seconds(&exclusion.damaged, &timeline),
    };
    let background = detected.background().unwrap();
    let colors = track_colors(&summary, background.width(), background.height()).unwrap();
    Drawing {
        video: video.to_path_buf(),
        detected,
        timeline,
        tracks,
        colors,
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
        let found = drawn(&source, self.colored(), &view(wanted), self.options.render).unwrap();
        let fingerprints = &found[0].fingerprints;
        Frozen {
            frames: fingerprints.len(),
            fingerprint: fingerprint::of_sequence(fingerprints.iter().copied()),
        }
    }

    fn colored(&self) -> Colored<'_> {
        Colored::new(&self.tracks, &self.colors).unwrap()
    }
}
