use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use render::clips::Rendered;

use super::{Done, Failure, Outcome, Panic, Rank, all_succeeded, run_all};
use crate::jobs::Job;
use crate::process::{Error, Processed};

fn jobs(count: usize) -> Vec<Job> {
    (0..count)
        .map(|index| Job {
            video: PathBuf::from(format!("video_{index}.mp4")),
            dest: PathBuf::from(format!("out/video_{index}")),
        })
        .collect()
}

fn processed() -> Processed {
    Processed {
        tracks: Vec::new(),
        csv: Some(PathBuf::from("tracks.csv")),
        kept: Vec::new(),
        summaries: Vec::new(),
        rendered: Rendered::default(),
    }
}

fn kinds(outcomes: &[Outcome]) -> Vec<&'static str> {
    outcomes
        .iter()
        .map(|outcome| match outcome {
            Ok(_) => "done",
            Err(Failure::Error(_)) => "error",
            Err(Failure::Panic(_)) => "panic",
        })
        .collect()
}

#[test]
fn a_panic_on_the_second_video_leaves_the_third_one_processed_and_reported() {
    let mut reported = Vec::new();
    let outcomes = run_all(
        &jobs(3),
        |job, _| {
            assert_ne!(job.video, PathBuf::from("video_1.mp4"), "forced panic");
            Ok(processed())
        },
        |job, _, result| {
            reported.push((job.video.clone(), result.is_ok()));
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(kinds(&outcomes), ["done", "panic", "done"]);
    assert_eq!(
        reported,
        [
            (PathBuf::from("video_0.mp4"), true),
            (PathBuf::from("video_1.mp4"), false),
            (PathBuf::from("video_2.mp4"), true)
        ]
    );
    assert!(matches!(&outcomes[1], Err(Failure::Panic(panic)) if panic.message.contains("forced panic")));
    assert!(matches!(outcomes[0], Ok(Done { tracks: 0 })));
}

#[test]
fn the_message_of_a_panic_is_kept_whether_it_is_a_literal_or_formatted() {
    let panicking = [
        |_: &Job, _: Rank| -> Result<Processed, Error> { std::panic::panic_any("literal") },
        |job: &Job, _: Rank| std::panic::panic_any(format!("formatted {}", job.video.display())),
    ];
    for (work, expected) in panicking.into_iter().zip(["literal", "formatted video_0.mp4"]) {
        let outcomes = run_all(&jobs(1), work, |_, _, _| Ok(())).unwrap();
        assert!(matches!(&outcomes[0], Err(Failure::Panic(panic)) if panic.message == expected));
    }
    let outcomes = run_all(&jobs(1), |_, _| std::panic::panic_any(7_u8), |_, _, _| Ok(())).unwrap();
    assert!(matches!(&outcomes[0], Err(Failure::Panic(panic)) if panic.message == "unknown panic"));
}

#[test]
fn a_failed_report_stops_the_loop_before_the_next_video() {
    let worked = AtomicUsize::new(0);
    let stopped = run_all(
        &jobs(3),
        |_, _| {
            worked.fetch_add(1, Ordering::SeqCst);
            Ok(processed())
        },
        |_, _, _| Err(io::Error::other("closed")),
    );
    assert!(stopped.is_err());
    assert_eq!(worked.into_inner(), 1);
}

#[test]
fn the_lot_succeeds_only_when_every_video_is_done() {
    assert!(all_succeeded(&[]));
    assert!(all_succeeded(&[Ok(Done { tracks: 2 })]));
    assert!(!all_succeeded(&[
        Ok(Done { tracks: 2 }),
        Err(Failure::Panic(Panic {
            message: "boom".to_owned(),
            location: None
        }))
    ]));
    let failed = Err(Failure::Error(Error::TooManyTracks {
        count: 301,
        limit: 300,
        annotated: false,
    }));
    assert!(!all_succeeded(&[failed]));
}

#[test]
fn each_video_is_given_its_rank_in_the_lot_to_work_and_to_report() {
    let mut reported = Vec::new();
    run_all(
        &jobs(3),
        |_, rank| {
            assert_eq!(rank.total, 3);
            Ok(processed())
        },
        |_, rank, _| {
            reported.push(rank);
            Ok(())
        },
    )
    .unwrap();
    let positions: Vec<usize> = reported.iter().map(|rank| rank.position).collect();
    assert_eq!(positions, [1, 2, 3]);
}
