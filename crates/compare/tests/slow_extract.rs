use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

use compare::case::{Case, EXTRACT};
use compare::comparison::{Comparison, compare, frames_path};
use compare::report::write_report;
use reference::fixture::fixtures;
use reference::frames::Frames;
use reference::location::committed;
use reference::recording::Recording;
use tempfile::TempDir;

struct Run {
    comparison: Comparison,
    written: PathBuf,
    _folder: TempDir,
}

#[cfg(test)]
fn run() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let folder = TempDir::new().unwrap();
        let comparison = compare(&Case::committed(EXTRACT), Some(folder.path())).unwrap();
        write_report(std::slice::from_ref(&comparison), &mut io::stdout().lock()).unwrap();
        Run {
            comparison,
            written: frames_path(folder.path(), &format!("{EXTRACT}.mp4")),
            _folder: folder,
        }
    })
}

fn committed_detections() -> PathBuf {
    frames_path(&fixtures().join("rust"), &format!("{EXTRACT}.mp4"))
}

#[test]
#[ignore = "slow"]
fn the_extract_agrees_with_the_reference_on_98_percent_of_its_detections_both_ways() {
    assert!(run().comparison.detections.holds());
}

#[test]
#[ignore = "slow"]
fn the_extract_gives_the_detections_committed_from_the_mac_bit_for_bit_on_every_system() {
    let found = fs::read_to_string(&run().written).unwrap();
    let expected = fs::read_to_string(committed_detections()).unwrap();

    let first_gap = found
        .lines()
        .zip(expected.lines())
        .find(|(found, expected)| found != expected);

    assert_eq!(first_gap, None);
    assert_eq!(found, expected);
}

#[test]
#[ignore = "slow"]
fn the_extract_has_the_timestamps_key_frames_and_decoding_errors_of_the_reference() {
    let recording = Recording::read(&committed(EXTRACT)).unwrap();

    let frames = Frames::read(&run().written).unwrap();

    assert_eq!(frames.entries(), recording.frames.entries());
}
