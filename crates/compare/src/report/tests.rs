use std::io;
use std::path::Path;

use super::{write_failure, write_report, write_same_as};
use crate::comparison::{Comparison, compared};
use crate::sameness::Sameness;
use crate::testing::{extract, frames_without_detections, nothing_detected, pass_finding, same_pass};

#[derive(Debug, thiserror::Error)]
#[error("cannot read the summary")]
struct Unreadable(#[source] io::Error);

fn report(comparisons: &[Comparison]) -> String {
    let mut out = Vec::new();
    write_report(comparisons, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn a_video_in_agreement_is_reported_with_its_inputs_its_totals_and_its_time() {
    let recording = extract();

    let text = report(&[compared(&recording.summary.video.name, &same_pass(&recording), &recording, 2.31).unwrap()]);

    assert!(text.contains(
        "video_092_original_3m24-4m05.mp4\tdecode errors: reference 0, current 0, differing 0\tdamaged spans 0 (0 frames)\n"
    ));
    assert!(text.contains("1 videos\tdiffering frames 0\tlost 0\tadded 0\n"));
    assert!(text.contains("1 videos\ttracks 4\tpaired 4 (agreeing 4)\tlost 0\tadded 0\n"));
    assert!(text.contains("video_092_original_3m24-4m05.mp4\t2.3 s\n"));
    assert!(text.ends_with("differing from the reference: 0\n"));
}

#[test]
fn a_video_that_differs_is_named_at_the_end_with_its_gaps() {
    let recording = extract();
    let empty = pass_finding(&recording, nothing_detected(&recording));

    let text = report(&[compared(&recording.summary.video.name, &empty, &recording, 1.0).unwrap()]);

    assert!(text.ends_with("differing from the reference: 1\n  video_092_original_3m24-4m05.mp4\n"));
    assert!(text.contains("lost reference track"));
    assert!(
        text.lines()
            .any(|line| line.starts_with("  video_092_original_3m24-4m05.mp4 ")
                && line.contains(" frames ")
                && line.contains("\tlost "))
    );
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
fn the_runs_compared_with_an_earlier_folder_are_listed_under_its_name_then_counted() {
    let recording = extract();
    let name = &recording.summary.video.name;
    let with = |current: &reference::frames::Frames| Comparison {
        same_as: Some(Sameness::of(name, &recording.frames, current, &[])),
        ..compared(name, &same_pass(&recording), &recording, 1.0).unwrap()
    };
    let without_frames = compared(name, &same_pass(&recording), &recording, 1.0).unwrap();
    let mut out = Vec::new();

    write_same_as(
        Path::new("out/first"),
        &[
            with(&recording.frames),
            with(&frames_without_detections(&recording)),
            without_frames,
        ],
        &mut out,
    )
    .unwrap();

    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "same as out/first, outside the damaged spans");
    assert!(lines[1].starts_with("video_092_original_3m24-4m05.mp4\tidentical\t"));
    assert!(lines[2].starts_with("video_092_original_3m24-4m05.mp4\tdiffers\toutside "));
    assert_eq!(lines[3], "2 videos: identical 1, differing 1, inputs differ 0");
}
