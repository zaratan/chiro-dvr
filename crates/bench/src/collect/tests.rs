use std::num::NonZeroUsize;
use std::path::PathBuf;

use engine::config::DetectConfig;
use media::decode::Threads;
use tempfile::TempDir;
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;
use types::frame::FrameIndex;

use super::{Error, Targets, collect};
use crate::config::MatchConfig;
use crate::matching::near;
use crate::metrics::{Judging, evaluate};
use crate::noisy::FRAMES;
use crate::run::Provenance;
use crate::synthetic::sampling::Sampling;
use crate::synthetic::{Class, Motion};
use crate::testing::{noisy_video, slow_target};

fn replayed(targets: Vec<crate::synthetic::Target>) -> Targets {
    Targets::Replayed {
        path: PathBuf::from("bats.jsonl.gz"),
        targets,
    }
}

#[test]
fn a_slow_dark_target_replayed_on_a_noisy_video_is_found_where_it_was_injected() {
    let folder = TempDir::new().unwrap();
    let video = noisy_video(&folder.path().join("noisy.mp4"));
    let target = slow_target(0, 10, 40);

    let collected = collect(
        &video,
        Threads::Automatic,
        &DetectConfig::default(),
        replayed(vec![target.clone()]),
    )
    .unwrap();
    let run = &collected.run;
    let evaluation = evaluate(run, Judging::default()).unwrap();

    assert_eq!(run.reference.detection_count(), 0);
    assert_eq!(run.injected.frame_count(), FRAMES);
    assert_eq!(run.targets[0].observations.len(), 40);
    for observation in &run.targets[0].observations {
        assert_eq!(Some(observation.at), target.position(observation.frame));
        assert!(observation.effective <= -59.0, "{observation:?}");
    }
    let near_target = run
        .injected
        .iter()
        .flat_map(|(_, found)| found)
        .filter(|detection| {
            target
                .position(detection.frame)
                .is_some_and(|at| near(detection, at, 3.0))
        })
        .count();
    assert!(near_target >= 30, "{near_target}");
    assert!(evaluation.targets[0].found, "{evaluation:?}");
    assert_eq!(evaluation.false_tracks, 0);
    assert_eq!(run.provenance, Provenance::Replayed(PathBuf::from("bats.jsonl.gz")));
}

#[test]
fn a_replayed_target_ending_beyond_the_video_is_refused_before_the_injected_detection() {
    let folder = TempDir::new().unwrap();
    let video = noisy_video(&folder.path().join("noisy.mp4"));

    let refused = collect(
        &video,
        Threads::Automatic,
        &DetectConfig::default(),
        replayed(vec![slow_target(3, 31, 30)]),
    )
    .unwrap_err();

    assert!(
        matches!(
            refused,
            Error::Beyond {
                id: 3,
                end: 60,
                frames: 60
            }
        ),
        "{refused:?}"
    );
    assert_eq!(
        refused.to_string(),
        "target 3 ends at frame 60, beyond the 60 frames of the video"
    );
}

#[test]
fn drawn_targets_are_placed_inside_the_video_and_their_drawing_is_recorded() {
    let folder = TempDir::new().unwrap();
    let video = noisy_video(&folder.path().join("noisy.mp4"));
    let sampling = Sampling {
        seed: 4,
        classes: vec![Class::new(-30.0, 1.5, Motion::Hunt).unwrap()],
        per_class: NonZeroUsize::new(2).unwrap(),
        min_distance: 3.0 * MatchConfig::default().radius(),
    };
    let targets = Targets::Drawn {
        sampling: sampling.clone(),
        track: TrackConfig::default(),
        stability: StabilityConfig::default(),
    };

    let run = collect(&video, Threads::Automatic, &DetectConfig::default(), targets)
        .unwrap()
        .run;

    assert!(run.targets.len() <= 2);
    assert!(
        run.targets
            .iter()
            .all(|placed| placed.target.end() < FrameIndex::new(FRAMES))
    );
    assert!(matches!(run.provenance, Provenance::Drawn { sampling: drawn, .. } if drawn == sampling));
}

#[test]
fn a_missing_video_is_reported_as_a_failed_reference_detection() {
    let folder = TempDir::new().unwrap();

    let refused = collect(
        &folder.path().join("absent.mp4"),
        Threads::Automatic,
        &DetectConfig::default(),
        replayed(Vec::new()),
    )
    .unwrap_err();

    assert!(matches!(refused, Error::Reference(_)), "{refused:?}");
}
