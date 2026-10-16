use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use files::params::Finish;
use render::clips::Rendered;

use super::{Failure, Ran, Rank, Row, State, run_all};
use crate::jobs::Job;
use crate::process::{Error, Extent, Processed};
use crate::reuse::Earlier;

fn jobs(count: usize) -> Vec<Job> {
    (0..count)
        .map(|index| Job {
            video: PathBuf::from(format!("video_{index}.mp4")),
            dest: PathBuf::from(format!("out/video_{index}")),
        })
        .collect()
}

fn processed() -> Ran {
    Ran::Processed(Processed {
        tracks: Vec::new(),
        extent: EXTENT,
        csv: Some(PathBuf::from("tracks.csv")),
        kept: Vec::new(),
        summaries: Vec::new(),
        rendered: Rendered::default(),
    })
}

const EXTENT: Extent = Extent {
    length_s: 12.0,
    not_analysed_s: 1.0,
};

fn kinds(rows: &[Row]) -> Vec<&'static str> {
    rows.iter()
        .map(|row| match row.state {
            State::Done { .. } => "done",
            State::TooManyTracks { .. } => "too many",
            State::Skipped(_) => "skipped",
            State::Failed(Failure::Error(_)) => "error",
            State::Failed(Failure::Panic(_)) => "panic",
        })
        .collect()
}

fn panic_message(row: &Row) -> Option<&str> {
    match &row.state {
        State::Failed(Failure::Panic(panic)) => Some(&panic.message),
        _ => None,
    }
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
    assert!(panic_message(&outcomes[1]).is_some_and(|message| message.contains("forced panic")));
    assert!(matches!(
        outcomes[0].state,
        State::Done {
            tracks: 0,
            extent: EXTENT,
            ..
        }
    ));
    assert_eq!(outcomes[2].job, jobs(3)[2]);
}

#[test]
fn the_message_of_a_panic_is_kept_whether_it_is_a_literal_or_formatted() {
    let panicking = [
        |_: &Job, _: Rank| -> Result<Ran, Error> { std::panic::panic_any("literal") },
        |job: &Job, _: Rank| std::panic::panic_any(format!("formatted {}", job.video.display())),
    ];
    for (work, expected) in panicking.into_iter().zip(["literal", "formatted video_0.mp4"]) {
        let outcomes = run_all(&jobs(1), work, |_, _, _| Ok(())).unwrap();
        assert_eq!(panic_message(&outcomes[0]), Some(expected));
    }
    let outcomes = run_all(&jobs(1), |_, _| std::panic::panic_any(7_u8), |_, _, _| Ok(())).unwrap();
    assert_eq!(panic_message(&outcomes[0]), Some("unknown panic"));
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
fn too_many_tracks_becomes_a_state_of_its_own_with_its_measures_and_the_other_errors_stay_failures() {
    let rows = run_all(
        &jobs(2),
        |_, rank| {
            if rank.position == 1 {
                Err(Error::TooManyTracks {
                    count: 301,
                    limit: 300,
                    annotated: true,
                    extent: EXTENT,
                    kept: Vec::new(),
                })
            } else {
                Err(Error::Progress(io::Error::other("closed")))
            }
        },
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert_eq!(kinds(&rows), ["too many", "error"]);
    assert!(matches!(
        rows[0].state,
        State::TooManyTracks {
            count: 301,
            limit: 300,
            annotated: true,
            extent: EXTENT,
            ..
        }
    ));
}

#[test]
fn the_compute_time_of_each_video_is_measured_whatever_its_end() {
    let pause = Duration::from_millis(10);
    let rows = run_all(
        &jobs(2),
        |_, rank| {
            thread::sleep(pause);
            assert_eq!(rank.position, 1, "forced panic");
            Ok(processed())
        },
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert_eq!(kinds(&rows), ["done", "panic"]);
    assert!(rows.iter().all(|row| row.elapsed >= pause), "{rows:?}");
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

#[test]
fn a_video_already_done_keeps_its_earlier_figures_and_is_reported_like_the_others() {
    let earlier = Earlier {
        finish: Finish::TooManyTracks,
        tracks: 412,
        limit: 300,
        extent: EXTENT,
    };
    let mut reported = Vec::new();
    let rows = run_all(
        &jobs(2),
        |_, rank| {
            if rank.position == 1 {
                Ok(Ran::Skipped(earlier))
            } else {
                Ok(processed())
            }
        },
        |_, _, result| {
            reported.push(matches!(result, Ok(Ran::Skipped(_))));
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(kinds(&rows), ["skipped", "done"]);
    assert_eq!(reported, [true, false]);
    assert!(matches!(rows[0].state, State::Skipped(found) if found == earlier));
}
