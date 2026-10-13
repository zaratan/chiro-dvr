use std::path::{Path, PathBuf};

use engine::config::DetectConfig;
use media::decode::Threads;
use reference::bench::{REPORT_FILE, TARGETS_FILE, read_report, read_targets, write_report, write_targets};
use serde_json::json;
use tempfile::TempDir;

use super::{Error, against};
use crate::collect::{Targets, collect};
use crate::folder;
use crate::metrics::{Judging, evaluate};
use crate::report::{Seconds, UNKNOWN_CODE, bench_report};
use crate::testing::{noisy_video, slow_target};

fn recorded(root: &Path) -> (PathBuf, PathBuf) {
    let video = noisy_video(&root.join("noisy.mp4"));
    let targets = Targets::Replayed {
        path: root.join("targets.jsonl.gz"),
        targets: vec![slow_target(0, 10, 40), slow_target(1, 5, 2)],
    };
    let run = collect(&video, Threads::Automatic, &DetectConfig::default(), targets)
        .unwrap()
        .run;
    let evaluation = evaluate(&run, Judging::default()).unwrap();
    let report = bench_report(
        &run,
        Judging::default(),
        &evaluation,
        UNKNOWN_CODE.to_owned(),
        Seconds::default(),
    )
    .unwrap();
    let bench = root.join("recorded");
    folder::write(&run, &bench).unwrap();
    write_report(&bench.join(REPORT_FILE), &report).unwrap();
    (video, bench)
}

#[test]
fn the_same_targets_replayed_with_the_same_settings_are_found_as_recorded_and_the_guard_passes() {
    let root = TempDir::new().unwrap();
    let (video, bench) = recorded(root.path());

    let checked = against(
        &video,
        Threads::Automatic,
        &DetectConfig::default(),
        Judging::default(),
        &bench,
    )
    .unwrap();

    let guard = &checked.guard;
    assert_eq!(guard.classes.len(), 1);
    assert_eq!(
        (guard.classes[0].n, guard.classes[0].v011_found, guard.classes[0].found),
        (1, 1, 1)
    );
    assert_eq!((guard.classes[0].lost.len(), guard.classes[0].gained.len()), (0, 0));
    assert!(guard.v011_only.is_empty() && guard.rust_only.is_empty() && guard.contrast_differs.is_empty());
    assert!(!guard.failed());
    assert_eq!(checked.evaluation.targets.len(), 1);
    assert_eq!(checked.evaluation.not_visible, 1);
    assert_eq!(checked.collected.run.targets.len(), 2);
}

#[test]
fn a_recorded_bench_made_with_other_settings_is_refused_with_the_settings_that_differ() {
    let root = TempDir::new().unwrap();
    let (video, bench) = recorded(root.path());
    let path = bench.join(REPORT_FILE);
    let mut report = read_report(&path).unwrap();
    report.detect["threshold"] = json!(13);
    write_report(&path, &report).unwrap();

    let refused = against(
        &video,
        Threads::Automatic,
        &DetectConfig::default(),
        Judging::default(),
        &bench,
    )
    .unwrap_err();

    assert!(matches!(refused, Error::Settings { key: "detect" }), "{refused:?}");
    assert_eq!(
        refused.to_string(),
        "the detect settings differ from those of the recorded bench"
    );
}

#[test]
fn recorded_settings_of_an_unknown_shape_are_refused_with_their_key() {
    let root = TempDir::new().unwrap();
    let (video, bench) = recorded(root.path());
    let path = bench.join(REPORT_FILE);
    let mut report = read_report(&path).unwrap();
    report.matching = json!({"radius": 24.0});
    write_report(&path, &report).unwrap();

    let refused = against(
        &video,
        Threads::Automatic,
        &DetectConfig::default(),
        Judging::default(),
        &bench,
    )
    .unwrap_err();

    assert!(
        matches!(refused, Error::RecordedSettings { key: "match", .. }),
        "{refused:?}"
    );
}

#[test]
fn a_folder_without_a_recorded_report_is_refused() {
    let root = TempDir::new().unwrap();

    let refused = against(
        &root.path().join("video.mp4"),
        Threads::Automatic,
        &DetectConfig::default(),
        Judging::default(),
        root.path(),
    )
    .unwrap_err();

    assert!(matches!(refused, Error::File(_)), "{refused:?}");
}

#[test]
fn the_contrast_recorded_with_the_targets_decides_their_visibility_in_the_guard() {
    let root = TempDir::new().unwrap();
    let (video, bench) = recorded(root.path());
    let path = bench.join(TARGETS_FILE);
    let mut lines = read_targets(&path).unwrap();
    for observation in &mut lines[0].effective {
        observation.1 = -4.0;
    }
    write_targets(&path, &lines).unwrap();

    let checked = against(
        &video,
        Threads::Automatic,
        &DetectConfig::default(),
        Judging::default(),
        &bench,
    )
    .unwrap();

    let guard = &checked.guard;
    assert_eq!(
        guard
            .contrast_differs
            .iter()
            .map(|change| change.id)
            .collect::<Vec<_>>(),
        [0]
    );
    assert_eq!(guard.v011_only.iter().map(|change| change.id).collect::<Vec<_>>(), [0]);
    assert_eq!(guard.classes, []);
    assert_eq!(checked.evaluation.targets.len(), 1);
}
