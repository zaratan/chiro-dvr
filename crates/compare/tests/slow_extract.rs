use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

use compare::agreement::Agreement;
use compare::case::{Case, EXTRACT};
use compare::comparison::{Comparison, Folders, compare, frames_path};
use compare::report::write_report;
use compare::sameness::Verdict;
use reference::fixture::fixtures;
use reference::frames::Frames;
use reference::location::{Reference, committed};
use reference::recording::Recording;
use tempfile::TempDir;

struct Run {
    comparison: Comparison,
    again: Comparison,
    written: Result<String, String>,
    frames: Result<Frames, String>,
}

#[cfg(test)]
fn run() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let folder = TempDir::new().unwrap();
        let comparison = compare(
            &Case::committed(Reference::V011, EXTRACT),
            &Folders {
                frames: Some(folder.path().to_path_buf()),
                same_as: None,
            },
        )
        .unwrap();
        write_report(std::slice::from_ref(&comparison), &mut io::stdout().lock()).unwrap();
        let again = compare(
            &Case::committed(Reference::V011, EXTRACT),
            &Folders {
                frames: None,
                same_as: Some(folder.path().to_path_buf()),
            },
        )
        .unwrap();
        let path = frames_path(folder.path(), &format!("{EXTRACT}.mp4"));
        Run {
            comparison,
            again,
            written: fs::read_to_string(&path).map_err(|error| error.to_string()),
            frames: Frames::read(&path).map_err(|error| error.to_string()),
        }
    })
}

const V011_RATE: f64 = 0.98;

fn within_the_v011_rate(detections: &Agreement) -> bool {
    detections.reference_rate() >= V011_RATE && detections.candidate_rate() >= V011_RATE
}

fn committed_detections() -> PathBuf {
    frames_path(&fixtures().join("rust"), &format!("{EXTRACT}.mp4"))
}

#[test]
#[ignore = "slow"]
fn the_extract_agrees_with_the_reference_on_98_percent_of_its_detections_both_ways() {
    assert!(within_the_v011_rate(&run().comparison.detections));
}

#[test]
#[ignore = "slow"]
fn the_extract_gives_the_detections_committed_from_the_mac_bit_for_bit_on_every_system() {
    let found = run().written.as_ref().unwrap();
    let expected = fs::read_to_string(committed_detections()).unwrap();

    let first_gap = found
        .lines()
        .zip(expected.lines())
        .find(|(found, expected)| found != expected);

    assert_eq!(first_gap, None);
    assert_eq!(found, &expected);
}

#[test]
#[ignore = "slow"]
fn the_extract_has_the_timestamps_key_frames_and_decoding_errors_of_the_reference() {
    let recording = Recording::read(&committed(Reference::V011, EXTRACT)).unwrap();

    assert_eq!(run().frames.as_ref().unwrap().entries(), recording.frames.entries());
}

#[test]
#[ignore = "slow"]
fn the_extract_detected_again_against_its_written_frames_is_the_same_through_the_command_path() {
    let again = run().again.same_as.as_ref().unwrap();

    assert!(again.holds(), "{again:?}");
    assert!(matches!(
        again.verdict(),
        Verdict::Compared {
            span_count: 0,
            differing_inside: 0,
            ..
        }
    ));
}
