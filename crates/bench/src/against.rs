use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use engine::config::DetectConfig;
use engine::settings::DetectSettings;
use media::decode::Threads;
use reference::bench::{
    BenchReport, INJECTED_FILE, REPORT_FILE, TARGETS_FILE, read_detections, read_report, read_targets,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracking::settings::{StabilitySettings, TrackSettings};

use crate::collect::{self, Collected, Targets, collect};
use crate::guard::{self, Guard, compare};
use crate::metrics::{Evaluation, Judging, evaluate, evaluate_with};
use crate::recorded;
use crate::synthetic::injection::Injected;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Against {
    pub collected: Collected,
    pub evaluation: Evaluation,
    pub guard: Guard,
    pub evaluation_time: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot read the recorded bench")]
    File(#[from] reference::error::Error),
    #[error("{} holds a target that cannot be replayed", path.display())]
    Target {
        path: PathBuf,
        #[source]
        source: recorded::Error,
    },
    #[error("the recorded bench was made on {recorded}, not on {}", replayed.display())]
    Video { recorded: String, replayed: PathBuf },
    #[error("the recorded bench has {recorded} frames, the replayed video {replayed}")]
    Frames { recorded: usize, replayed: usize },
    #[error("the {key} settings differ from those of the recorded bench")]
    Settings { key: &'static str },
    #[error("the {key} settings of the recorded bench cannot be read")]
    RecordedSettings {
        key: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Collect(#[from] collect::Error),
    #[error("the injected run cannot be tracked")]
    Tracking(#[from] tracking::error::Error),
    #[error("the recorded report does not match its targets")]
    Guard(#[from] guard::Error),
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchSettings {
    radius: f64,
    purity: f64,
}

pub fn against(
    video: &Path,
    threads: Threads,
    config: &DetectConfig,
    settings: Judging,
    folder: &Path,
) -> Result<Against, Error> {
    let report = read_report(&folder.join(REPORT_FILE))?;
    same_video(&report, video)?;
    same_settings(&report, config, settings)?;
    let recorded_frames = read_detections(&folder.join(INJECTED_FILE))?.frame_count();
    let path = folder.join(TARGETS_FILE);
    let recorded: Vec<Injected> = recorded::all_injected(&read_targets(&path)?).map_err(|source| Error::Target {
        path: path.clone(),
        source,
    })?;
    let targets = recorded.iter().map(|placed| placed.target.clone()).collect();
    let collected = collect(video, threads, config, Targets::Replayed { path, targets })?;
    same_frames(recorded_frames, collected.run.reference.frame_count())?;
    let started = Instant::now();
    let evaluation = evaluate(&collected.run, settings)?;
    let evaluation_time = started.elapsed();
    let judged = evaluate_with(&collected.run, &recorded, settings)?;
    let replayed: Vec<_> = recorded.into_iter().map(|placed| placed.target).collect();
    let guard = compare(&judged, &evaluation, &report, &replayed)?;
    Ok(Against {
        collected,
        evaluation,
        guard,
        evaluation_time,
    })
}

fn same_video(report: &BenchReport, video: &Path) -> Result<(), Error> {
    let recorded = Path::new(&report.video).file_name();
    if recorded.is_some() && recorded == video.file_name() {
        Ok(())
    } else {
        Err(Error::Video {
            recorded: report.video.clone(),
            replayed: video.to_path_buf(),
        })
    }
}

fn same_frames(recorded: usize, replayed: usize) -> Result<(), Error> {
    if recorded == replayed {
        Ok(())
    } else {
        Err(Error::Frames { recorded, replayed })
    }
}

fn same_settings(report: &BenchReport, config: &DetectConfig, settings: Judging) -> Result<(), Error> {
    same(&report.detect, &DetectSettings::from(config), "detect")?;
    same(&report.track, &TrackSettings::from(settings.track), "track")?;
    same(
        &report.stability,
        &StabilitySettings::from(settings.stability),
        "stability",
    )?;
    let matching = MatchSettings {
        radius: settings.matching.radius(),
        purity: settings.matching.purity(),
    };
    same(&report.matching, &matching, "match")
}

fn same<T: DeserializeOwned + PartialEq>(recorded: &Value, ours: &T, key: &'static str) -> Result<(), Error> {
    let read = T::deserialize(recorded).map_err(|source| Error::RecordedSettings { key, source })?;
    if read == *ours {
        Ok(())
    } else {
        Err(Error::Settings { key })
    }
}
