use std::num::NonZeroUsize;
use std::time::Duration;

use reference::bench::{read_report, write_report};
use serde_json::json;
use tempfile::TempDir;
use tracking::settings::{StabilitySettings, TrackSettings};
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

use super::{Seconds, UNKNOWN_CODE, bench_report, code_version, code_version_in};
use crate::metrics::{Judging, evaluate};
use crate::run::{Provenance, Run};
use crate::synthetic::sampling::Sampling;
use crate::synthetic::{Class, Motion};
use crate::testing::{along, observed, on, run};

fn found_run() -> Run {
    let target = along(0, 10, 20, 180.0);
    run(
        vec![observed(target.clone(), -40.0)],
        on(&target),
        Vec::new(),
        Vec::new(),
    )
}

fn seconds() -> Seconds {
    Seconds {
        reference: Some(Duration::from_millis(1500)),
        injected: Some(Duration::from_millis(2000)),
        evaluation: Duration::from_millis(500),
    }
}

#[test]
fn a_written_report_reads_back_with_the_keys_of_the_v011_and_the_two_added_ones() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("bench.json");
    let run = found_run();
    let evaluation = evaluate(&run, Judging::default()).unwrap();
    let report = bench_report(&run, Judging::default(), &evaluation, "v0.2.0".to_owned(), seconds()).unwrap();

    write_report(&path, &report).unwrap();

    let read = read_report(&path).unwrap();
    assert_eq!(read, report);
    assert_eq!((read.video.as_str(), read.code.as_str()), ("video.mp4", "v0.2.0"));
    assert_eq!(read.bats[0].id, 0);
    assert!(read.bats[0].found);
    assert_eq!(read.summary[0].n, 1);
    assert_eq!(read.injected_tracks, 1);
    assert_eq!(read.track, json!(TrackSettings::from(TrackConfig::default())));
    assert_eq!(
        read.stability,
        json!(StabilitySettings::from(StabilityConfig::default()))
    );
    assert_eq!(read.matching, json!({"radius": 24.0, "purity": 0.5}));
    assert_eq!(read.detect["work_width"], 960);
    assert_eq!(read.sampling, json!(null));
    assert_eq!(read.added["targets"], json!({"replayed": "bats.jsonl.gz"}));
    assert_eq!(
        read.added["seconds"],
        json!({"reference": 1.5, "injected": 2.0, "evaluation": 0.5, "total": 4.0})
    );
}

#[test]
fn drawn_targets_report_their_sampling_and_seed() {
    let mut run = found_run();
    run.provenance = Provenance::Drawn {
        sampling: Sampling {
            seed: 7,
            classes: vec![Class::new(-30.0, 1.5, Motion::Circle).unwrap()],
            per_class: NonZeroUsize::new(30).unwrap(),
            min_distance: 72.0,
        },
        track: TrackSettings::from(TrackConfig::default()),
        stability: StabilitySettings::from(StabilityConfig::default()),
    };
    let evaluation = evaluate(&run, Judging::default()).unwrap();

    let report = bench_report(
        &run,
        Judging::default(),
        &evaluation,
        UNKNOWN_CODE.to_owned(),
        seconds(),
    )
    .unwrap();

    assert_eq!(
        report.sampling,
        json!({"seed": 7, "classes": [{"amplitude": -30.0, "sigma": 1.5, "motion": "circle"}],
               "per_class": 30, "min_distance": 72.0})
    );
    assert_eq!(report.added["targets"], json!({"drawn": {"seed": 7}}));
}

#[test]
fn a_replayed_run_reports_only_the_time_of_its_evaluation() {
    let run = found_run();
    let evaluation = evaluate(&run, Judging::default()).unwrap();
    let replayed = Seconds {
        evaluation: Duration::from_millis(250),
        ..Seconds::default()
    };

    let report = bench_report(&run, Judging::default(), &evaluation, UNKNOWN_CODE.to_owned(), replayed).unwrap();

    assert_eq!(report.added["seconds"], json!({"evaluation": 0.25, "total": 0.25}));
}

#[test]
fn the_code_is_described_by_git_or_unknown_outside_a_repository() {
    let outside = TempDir::new().unwrap();

    assert_eq!(code_version_in(outside.path()), UNKNOWN_CODE);
    assert_ne!(code_version(), "");
}
