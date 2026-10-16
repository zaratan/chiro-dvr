use std::path::{Path, PathBuf};
use std::time::Duration;

use files::params::Finish;
use files::previous::Kept;

use super::Tally;
use crate::jobs::{Job, Reason, Refused};
use crate::outcome::{Failure, Panic, Row, State};
use crate::process::Extent;
use crate::reuse::Earlier;

const EXTENT: Extent = Extent {
    length_s: 10.0,
    not_analysed_s: 0.0,
};

fn row(dest: &str, state: State) -> Row {
    Row {
        job: Job {
            video: PathBuf::from("v.mp4"),
            dest: PathBuf::from(dest),
        },
        elapsed: Duration::ZERO,
        state,
    }
}

fn kept(path: &str) -> Vec<Kept> {
    vec![Kept::Left {
        path: PathBuf::from(path),
        ours: false,
    }]
}

fn evening() -> Tally {
    let panic = Panic {
        message: "boom".to_owned(),
        location: None,
    };
    Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![
            row(
                "a/batdetect/1",
                State::Done {
                    tracks: 3,
                    extent: EXTENT,
                    kept: kept("a/batdetect/1/1_tracks.csv"),
                },
            ),
            row("b/batdetect/1", State::Failed(Failure::Panic(panic))),
            row(
                "a/batdetect/2",
                State::TooManyTracks {
                    count: 9,
                    limit: 5,
                    annotated: false,
                    extent: EXTENT,
                    kept: kept("a/batdetect/2/2_pistes.csv"),
                },
            ),
        ],
    }
}

fn dests<'a>(rows: impl Iterator<Item = &'a Row>) -> Vec<&'a Path> {
    rows.map(|row| row.job.dest.as_path()).collect()
}

#[test]
fn the_folders_come_in_the_order_they_first_appear_and_a_folder_of_failures_never_written_is_not_titled() {
    let done = || State::Done {
        tracks: 1,
        extent: EXTENT,
        kept: Vec::new(),
    };
    let night = Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![
            row("c/batdetect/1", done()),
            row("a/batdetect/1", done()),
            row("c/batdetect/2", done()),
        ],
    };
    assert_eq!(night.roots(), [Path::new("c/batdetect"), Path::new("a/batdetect")]);
    let tally = evening();
    assert_eq!(tally.roots(), [Path::new("a/batdetect")]);
    assert_eq!(
        dests(tally.rows_in(Path::new("a/batdetect"))),
        [Path::new("a/batdetect/1"), Path::new("a/batdetect/2")]
    );
}

#[test]
fn each_state_is_found_apart_and_too_many_tracks_is_not_a_failure() {
    let tally = evening();
    assert_eq!(dests(tally.done()), [Path::new("a/batdetect/1")]);
    assert_eq!(
        dests(tally.too_many().map(|(row, ..)| row)),
        [Path::new("a/batdetect/2")]
    );
    assert_eq!(
        tally
            .too_many()
            .map(|(_, count, limit)| (count, limit))
            .collect::<Vec<_>>(),
        [(9, 5)]
    );
    assert_eq!(dests(tally.failed().map(|(row, _)| row)), [Path::new("b/batdetect/1")]);
    assert!(!tally.all_done());
}

#[test]
fn the_tables_left_as_they_are_come_from_done_videos_and_from_videos_with_too_many_tracks() {
    let tally = evening();
    let kept: Vec<&Path> = tally
        .kept()
        .map(|(_, table)| match table {
            Kept::Left { path, .. } | Kept::SetAside { path, .. } => path.as_path(),
        })
        .collect();
    assert_eq!(
        kept,
        [
            Path::new("a/batdetect/1/1_tracks.csv"),
            Path::new("a/batdetect/2/2_pistes.csv")
        ]
    );
}

#[test]
fn a_lot_without_failure_nor_too_many_tracks_is_all_done() {
    let tally = Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![row(
            "a/batdetect/1",
            State::Done {
                tracks: 0,
                extent: EXTENT,
                kept: Vec::new(),
            },
        )],
    };
    assert!(tally.all_done());
}

#[test]
fn a_refused_input_keeps_the_evening_from_counting_as_all_done() {
    let refused = Refused {
        path: PathBuf::from("absente.mp4"),
        reason: Reason::Missing,
    };
    assert!(
        Tally {
            refused: Vec::new(),
            unwritten: Vec::new(),
            rows: Vec::new()
        }
        .all_done()
    );
    assert!(
        !Tally {
            refused: vec![refused],
            unwritten: Vec::new(),
            rows: Vec::new()
        }
        .all_done()
    );
}

#[test]
fn a_folder_of_failed_videos_only_is_titled_only_if_it_exists() {
    let night = tempfile::tempdir().unwrap();
    let written = night.path().join("batdetect");
    std::fs::create_dir(&written).unwrap();
    let failed = |dest: PathBuf| Row {
        job: Job {
            video: PathBuf::from("v.mp4"),
            dest,
        },
        elapsed: Duration::ZERO,
        state: State::Failed(Failure::Panic(Panic {
            message: "boom".to_owned(),
            location: None,
        })),
    };
    let tally = Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![
            failed(night.path().join("jamais/batdetect/1")),
            failed(written.join("2")),
        ],
    };
    assert_eq!(tally.roots(), [written.as_path()]);
    assert_eq!(tally.failed().count(), 2);
}

fn skipped(dest: &str, finish: Finish) -> Row {
    row(
        dest,
        State::Skipped(Earlier {
            finish,
            tracks: 7,
            limit: 5,
            extent: EXTENT,
        }),
    )
}

#[test]
fn a_video_already_done_counts_apart_and_one_left_with_too_many_tracks_still_counts_as_too_many() {
    let tally = Tally {
        unwritten: Vec::new(),
        refused: Vec::new(),
        rows: vec![
            skipped("a/batdetect/1", Finish::Done),
            skipped("a/batdetect/2", Finish::TooManyTracks),
        ],
    };
    assert_eq!(dests(tally.already_done()), [Path::new("a/batdetect/1")]);
    assert_eq!(tally.done().count(), 0);
    let too_many: Vec<(&Path, usize, usize)> = tally
        .too_many()
        .map(|(row, count, limit)| (row.job.dest.as_path(), count, limit))
        .collect();
    assert_eq!(too_many, [(Path::new("a/batdetect/2"), 7, 5)]);
    assert_eq!((tally.failed().count(), tally.kept().count()), (0, 0));
    assert!(!tally.all_done());
}

#[test]
fn an_evening_of_videos_all_already_done_ends_well() {
    let tally = Tally {
        unwritten: Vec::new(),
        refused: Vec::new(),
        rows: vec![
            skipped("a/batdetect/1", Finish::Done),
            skipped("b/batdetect/1", Finish::Done),
        ],
    };
    assert!(tally.all_done());
    assert_eq!(tally.roots().len(), 2);
}
