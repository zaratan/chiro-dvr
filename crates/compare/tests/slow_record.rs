use std::fs;

use compare::case::{Case, EXTRACT};
use compare::comparison::frames_path;
use compare::provenance::Build;
use compare::recorder::{Places, record};
use reference::fixture::fixtures;
use reference::location::Reference;
use reference::recording::Recording;
use tempfile::TempDir;

#[test]
#[ignore = "slow"]
fn the_extract_is_recorded_with_its_real_stream_and_the_detections_committed_from_the_mac_bit_for_bit() {
    let out = TempDir::new().unwrap();
    let build = Build::clean("v0.2.0-1-g28d5526", "28d5526d0b2c", "rustc 1.90.0").unwrap();
    let places = Places {
        out: out.path(),
        same_as: Some(&fixtures().join("rust")),
    };

    let recorded = record(&Case::committed(Reference::V011, EXTRACT), &build, &places).unwrap();

    let read = Recording::read(&recorded.directory).unwrap();
    let written = fs::read_to_string(recorded.directory.join(&read.summary.files.frames)).unwrap();
    let committed = fs::read_to_string(frames_path(&fixtures().join("rust"), &recorded.name)).unwrap();
    assert_eq!(written, committed);
    assert!(recorded.holds());
    assert_eq!(read.summary.counts.tracks, 4);
}
