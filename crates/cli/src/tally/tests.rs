use std::path::{Path, PathBuf};
use std::time::Duration;

use files::previous::Kept;

use super::Tally;
use crate::jobs::{Job, Reason, Refused};
use crate::outcome::{Failure, Panic, Row, State};
use crate::process::Extent;

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
    vec![Kept {
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
fn the_folders_come_in_the_order_they_first_appear_and_each_keeps_its_videos_in_the_order_of_the_lot() {
    let tally = evening();
    assert_eq!(tally.roots(), [Path::new("a/batdetect"), Path::new("b/batdetect")]);
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
    let kept: Vec<&Path> = tally.kept().map(|(_, table)| table.path.as_path()).collect();
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
            rows: Vec::new()
        }
        .all_done()
    );
    assert!(
        !Tally {
            refused: vec![refused],
            rows: Vec::new()
        }
        .all_done()
    );
}
