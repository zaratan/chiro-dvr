use std::fs;
use std::num::NonZeroUsize;

use engine::config::DetectConfig;
use media::decode::Threads;
use serde_json::{Value, json};
use tempfile::TempDir;
use tracking::settings::{StabilitySettings, TrackSettings};
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;
use types::frame::FrameIndex;
use types::span::Span;

use super::{Error, RUN_FILE, read, write};
use crate::collect::{Targets, collect};
use crate::metrics::{Judging, evaluate};
use crate::run::{Provenance, Run};
use crate::synthetic::sampling::Sampling;
use crate::synthetic::{Class, Motion};
use crate::testing::{along, noisy_video, observed, on, run, slow_target};

type Case = (fn(&mut Value), fn(&Error) -> bool);

fn drawn_run() -> Run {
    let target = along(0, 10, 20, 180.0);
    let damaged = vec![Span::new(FrameIndex::new(60), FrameIndex::new(74)).unwrap()];
    let mut drawn = run(vec![observed(target.clone(), -40.0)], on(&target), Vec::new(), damaged);
    drawn.provenance = Provenance::Drawn {
        sampling: Sampling {
            seed: 9,
            classes: vec![Class::new(-40.0, 3.0, Motion::Pass).unwrap()],
            per_class: NonZeroUsize::MIN,
            min_distance: 72.0,
        },
        track: TrackSettings::from(TrackConfig::default()),
        stability: StabilitySettings::from(StabilityConfig::default()),
    };
    drawn
}

fn written(run: &Run) -> TempDir {
    let folder = TempDir::new().unwrap();
    write(run, &folder.path().join("run")).unwrap();
    folder
}

fn edited(edit: impl FnOnce(&mut Value)) -> Error {
    let folder = written(&drawn_run());
    let path = folder.path().join("run").join(RUN_FILE);
    let mut value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(&path, value.to_string()).unwrap();
    read(&folder.path().join("run")).unwrap_err()
}

#[test]
fn a_run_with_drawn_targets_reads_back_as_it_was_written() {
    let drawn = drawn_run();

    let folder = written(&drawn);

    assert_eq!(read(&folder.path().join("run")).unwrap(), drawn);
}

#[test]
fn a_collected_run_written_then_read_gives_the_same_evaluation() {
    let folder = TempDir::new().unwrap();
    let video = noisy_video(&folder.path().join("noisy.mp4"));
    let targets = Targets::Replayed {
        path: folder.path().join("bats.jsonl.gz"),
        targets: vec![slow_target(0, 10, 40)],
    };
    let collected = collect(&video, Threads::Automatic, &DetectConfig::default(), targets)
        .unwrap()
        .run;

    write(&collected, &folder.path().join("run")).unwrap();
    let replayed = read(&folder.path().join("run")).unwrap();

    assert_eq!(replayed, collected);
    assert_eq!(
        evaluate(&replayed, Judging::default()).unwrap(),
        evaluate(&collected, Judging::default()).unwrap()
    );
}

#[test]
fn a_damaged_run_description_is_refused_with_what_is_wrong() {
    let cases: [Case; 10] = [
        (
            |run| run["format_version"] = json!(2),
            |error| matches!(error, Error::Version { found: 2, .. }),
        ),
        (
            |run| run["width"] = json!(0),
            |error| matches!(error, Error::Size { width: 0, .. }),
        ),
        (
            |run| run["frame_rate"] = json!("-30/1"),
            |error| matches!(error, Error::Rate { .. }),
        ),
        (
            |run| run["damaged_spans"] = json!([[8, 3]]),
            |error| matches!(error, Error::Span { .. }),
        ),
        (
            |run| run["half_window"] = json!(0),
            |error| matches!(error, Error::HalfWindow { .. }),
        ),
        (
            |run| run["targets"]["drawn"]["classes"][0]["sigma"] = json!(0.0),
            |error| matches!(error, Error::Class { .. }),
        ),
        (
            |run| run["targets"]["drawn"]["classes"][0]["motion"] = json!("glide"),
            |error| matches!(error, Error::Motion { .. }),
        ),
        (
            |run| run["targets"]["drawn"]["per_class"] = json!(0),
            |error| matches!(error, Error::PerClass { .. }),
        ),
        (
            |run| run["seed"] = json!(3),
            |error| matches!(error, Error::Json { .. }),
        ),
        (
            |run| run["detect"]["threshold"] = json!(-1.0),
            |error| matches!(error, Error::Detect { .. }),
        ),
    ];
    for (edit, expected) in cases {
        let refused = edited(edit);

        assert!(expected(&refused), "{refused:?}");
        assert!(refused.to_string().contains(RUN_FILE), "{refused}");
    }
}

#[test]
fn a_folder_without_a_run_description_is_refused_with_its_path() {
    let folder = TempDir::new().unwrap();

    let refused = read(folder.path()).unwrap_err();

    assert!(matches!(refused, Error::Read { .. }), "{refused:?}");
    assert_eq!(
        refused.to_string(),
        format!("cannot read {}", folder.path().join(RUN_FILE).display())
    );
}

#[test]
fn a_run_whose_targets_file_holds_a_contrast_outside_a_flight_is_refused() {
    let folder = written(&drawn_run());
    let targets = folder.path().join("run").join(reference::bench::TARGETS_FILE);
    let mut lines = reference::bench::read_targets(&targets).unwrap();
    lines[0].effective.push((99, -40.0));
    reference::bench::write_targets(&targets, &lines).unwrap();

    let refused = read(&folder.path().join("run")).unwrap_err();

    assert!(matches!(refused, Error::Target { .. }), "{refused:?}");
}

#[test]
fn a_run_cannot_be_written_where_a_file_stands() {
    let folder = TempDir::new().unwrap();
    let blocking = folder.path().join("run");
    fs::write(&blocking, "").unwrap();

    let refused = write(&drawn_run(), &blocking).unwrap_err();

    assert!(matches!(refused, Error::Folder { .. }), "{refused:?}");
}

#[test]
fn a_run_whose_target_ends_beyond_its_frames_is_refused() {
    let folder = written(&drawn_run());
    let targets = folder.path().join("run").join(reference::bench::TARGETS_FILE);
    let mut lines = reference::bench::read_targets(&targets).unwrap();
    lines[0].start_frame = 95;
    lines[0].effective.clear();
    reference::bench::write_targets(&targets, &lines).unwrap();

    let refused = read(&folder.path().join("run")).unwrap_err();

    assert!(
        matches!(refused, Error::Beyond { id: 0, frames: 100, .. }),
        "{refused:?}"
    );
}
