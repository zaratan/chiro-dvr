use std::io;
use std::path::{Path, PathBuf};

use reference::location::committed;
use reference::recording::Recording;
use types::detection::FrameDetections;

use super::{write_failure, write_refusals, write_report};
use crate::case::EXTRACT;
use crate::comparison::{Comparison, compared};
use crate::outcome::Refusal;

#[derive(Debug, thiserror::Error)]
#[error("cannot read the summary")]
struct Unreadable(#[source] io::Error);

fn report(comparisons: &[Comparison]) -> String {
    let mut out = Vec::new();
    write_report(comparisons, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn a_video_in_agreement_is_reported_with_its_totals_and_its_time() {
    let recording = Recording::read(&committed(EXTRACT)).unwrap();

    let text = report(&[compared(
        &recording.summary.video.name,
        recording.frames.detections(),
        &recording,
        2.31,
    )
    .unwrap()]);

    assert!(text.contains("1 videos\ttracks 4\tpaired 4 (agreeing 4)\tlost 0\tadded 0\n"));
    assert!(text.contains("video_092_original_3m24-4m05.mp4\t2.3 s\n"));
    assert!(text.ends_with("under 98 % either way: 0\n"));
}

#[test]
fn a_video_under_98_percent_is_named_at_the_end_with_its_gaps() {
    let recording = Recording::read(&committed(EXTRACT)).unwrap();
    let empty = FrameDetections::new(vec![Vec::new(); recording.frames.detections().frame_count()]).unwrap();

    let text = report(&[compared(&recording.summary.video.name, &empty, &recording, 1.0).unwrap()]);

    assert!(text.ends_with("under 98 % either way: 1\n  video_092_original_3m24-4m05.mp4\n"));
    assert!(text.contains("lost reference track"));
    assert!(text.contains("frames "));
}

#[test]
fn a_failure_names_its_recording_then_each_of_its_causes() {
    let error = Unreadable(io::Error::other("disk gone"));
    let mut out = Vec::new();

    write_failure(Path::new("data/video_089"), &error, &mut out).unwrap();

    assert_eq!(
        String::from_utf8(out).unwrap(),
        "data/video_089: cannot read the summary\n  disk gone\n"
    );
}

#[test]
fn each_refused_case_is_named_by_its_folder_with_what_the_engine_cannot_do_yet() {
    let mut out = Vec::new();
    let refusals = [
        Refusal {
            recording: PathBuf::from("fixtures/reference/settings/osd"),
            reason: "osd_regions: masks are not handled, 1 given".to_owned(),
        },
        Refusal {
            recording: PathBuf::from("fixtures/reference/settings/step_1"),
            reason: "more samples".to_owned(),
        },
    ];

    write_refusals(&refusals, &mut out).unwrap();

    assert_eq!(
        String::from_utf8(out).unwrap(),
        "refused, the engine cannot handle them yet: 2\n  osd: osd_regions: masks are not handled, 1 given\n  step_1: more samples\n"
    );
}

#[test]
fn no_refusal_still_says_so() {
    let mut out = Vec::new();

    write_refusals(&[], &mut out).unwrap();

    assert_eq!(
        String::from_utf8(out).unwrap(),
        "refused, the engine cannot handle them yet: 0\n"
    );
}
