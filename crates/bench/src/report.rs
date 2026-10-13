use std::path::Path;
use std::process::Command;
use std::time::Duration;

use reference::bench::{BenchReport, ReportedClass, ReportedTarget};
use serde::Serialize;
use serde_json::Map;
use tracking::settings::{StabilitySettings, TrackSettings};

use crate::metrics::{Evaluation, Judging, TargetResult};
use crate::run::{Provenance, Run};
use crate::stats::{ClassSummary, summarize};

#[cfg(test)]
mod tests;

pub(crate) const UNKNOWN_CODE: &str = "unknown";

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Seconds {
    pub reference: Option<Duration>,
    pub injected: Option<Duration>,
    pub evaluation: Duration,
}

pub fn code_version() -> String {
    code_version_in(Path::new("."))
}

fn code_version_in(folder: &Path) -> String {
    Command::new("git")
        .args(["describe", "--always", "--dirty"])
        .current_dir(folder)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| UNKNOWN_CODE.to_owned())
}

#[derive(Serialize)]
struct MatchWritten {
    radius: f64,
    purity: f64,
}

#[derive(Serialize)]
struct ClassWritten {
    amplitude: f64,
    sigma: f64,
    motion: &'static str,
}

#[derive(Serialize)]
struct SamplingWritten {
    seed: u64,
    classes: Vec<ClassWritten>,
    per_class: usize,
    min_distance: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum TargetsWritten {
    Drawn { seed: u64 },
    Replayed(String),
}

#[derive(Serialize)]
struct SecondsWritten {
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    injected: Option<f64>,
    evaluation: f64,
    total: f64,
}

pub fn bench_report(
    run: &Run,
    settings: Judging,
    evaluation: &Evaluation,
    code: String,
    seconds: Seconds,
) -> Result<BenchReport, serde_json::Error> {
    let matching = MatchWritten {
        radius: settings.matching.radius(),
        purity: settings.matching.purity(),
    };
    Ok(BenchReport {
        video: run.video.path.display().to_string(),
        code,
        detect: serde_json::to_value(&run.detect)?,
        track: serde_json::to_value(TrackSettings::from(settings.track))?,
        stability: serde_json::to_value(StabilitySettings::from(settings.stability))?,
        matching: serde_json::to_value(matching)?,
        sampling: serde_json::to_value(sampling(&run.provenance))?,
        summary: summarize(evaluation).iter().map(reported_class).collect(),
        not_visible: evaluation.not_visible,
        false_tracks: evaluation.false_tracks,
        reference_tracks: evaluation.reference_tracks,
        injected_tracks: evaluation.injected_tracks,
        bats: evaluation.targets.iter().map(reported_target).collect(),
        added: Map::from_iter([
            ("targets".to_owned(), serde_json::to_value(targets(&run.provenance))?),
            ("seconds".to_owned(), serde_json::to_value(seconds_written(seconds))?),
        ]),
    })
}

fn sampling(provenance: &Provenance) -> Option<SamplingWritten> {
    match provenance {
        Provenance::Drawn { sampling, .. } => Some(SamplingWritten {
            seed: sampling.seed,
            classes: sampling
                .classes
                .iter()
                .map(|class| ClassWritten {
                    amplitude: class.amplitude(),
                    sigma: class.sigma(),
                    motion: class.motion().name(),
                })
                .collect(),
            per_class: sampling.per_class.get(),
            min_distance: sampling.min_distance,
        }),
        Provenance::Replayed(_) => None,
    }
}

fn targets(provenance: &Provenance) -> TargetsWritten {
    match provenance {
        Provenance::Drawn { sampling, .. } => TargetsWritten::Drawn { seed: sampling.seed },
        Provenance::Replayed(path) => TargetsWritten::Replayed(path.display().to_string()),
    }
}

fn seconds_written(seconds: Seconds) -> SecondsWritten {
    let detections: Duration = [seconds.reference, seconds.injected].into_iter().flatten().sum();
    SecondsWritten {
        reference: seconds.reference.as_ref().map(Duration::as_secs_f64),
        injected: seconds.injected.as_ref().map(Duration::as_secs_f64),
        evaluation: seconds.evaluation.as_secs_f64(),
        total: (detections + seconds.evaluation).as_secs_f64(),
    }
}
fn reported_class(row: &ClassSummary) -> ReportedClass {
    ReportedClass {
        motion: row.class.motion().name().to_owned(),
        amplitude: row.class.amplitude(),
        sigma: row.class.sigma(),
        n: row.n,
        found: row.found,
        found_ci95: [row.found_ci95.0, row.found_ci95.1],
        median_completeness: row.median_completeness,
        median_start_delay: row.median_start_delay,
        median_end_early: row.median_end_early,
        mean_fragments: row.mean_fragments,
        median_effective: row.median_effective,
    }
}

fn reported_target(result: &TargetResult) -> ReportedTarget {
    ReportedTarget {
        id: result.id,
        motion: result.class.motion().name().to_owned(),
        amplitude: result.class.amplitude(),
        sigma: result.class.sigma(),
        visible_frames: result.visible_frames,
        found: result.found,
        completeness: result.completeness,
        start_delay: result.start_delay,
        end_early: result.end_early,
        fragments: result.fragments,
        median_effective: result.median_effective,
    }
}
