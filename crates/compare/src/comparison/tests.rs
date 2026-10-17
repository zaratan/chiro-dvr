use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};

use reference::frames::Frames;
use reference::location::{Reference, committed};
use reference::recording::Recording;
use tempfile::TempDir;
use types::detection::FrameDetections;
use types::frame::{FrameIndex, Pts};
use types::losses::EdgeLosses;
use types::span::Span;

use super::{FRAMES_FILE, Folders, compare, compare_all, compared, each_in_order, frames_path};
use crate::case::{Case, EXTRACT};
use crate::error::Error;
use crate::frames::Pass;
use crate::inputs::{DecodeErrors, Field, InputGap};
use crate::testing::{extract, same_pass};
use crate::tracks::TrackCounts;

fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

fn with_entries(pass: &Pass, edit: impl Fn(&mut [reference::frames::FrameEntry])) -> Pass {
    let mut entries = pass.frames.entries().to_vec();
    edit(&mut entries);
    Pass {
        frames: Frames::new(entries, pass.frames.detections().clone()).unwrap(),
        reports: pass.reports.clone(),
    }
}

#[test]
fn the_frames_of_a_video_go_to_a_folder_named_after_it() {
    assert_eq!(
        frames_path(Path::new("out"), "video_092_original.mp4"),
        PathBuf::from("out/video_092_original").join(FRAMES_FILE)
    );
    assert_eq!(
        frames_path(Path::new("out"), "threshold_25"),
        PathBuf::from("out/threshold_25").join(FRAMES_FILE)
    );
}

#[test]
fn a_recording_folder_without_a_summary_is_reported() {
    let folder = TempDir::new().unwrap();

    let error = compare(&Case::recorded(folder.path().to_path_buf()), &Folders::default()).unwrap_err();

    assert!(matches!(error, Error::Reference(_)));
}

#[test]
fn a_recording_whose_video_is_missing_is_reported_before_any_detection() {
    let folder = TempDir::new().unwrap();
    let mut recording = extract();
    recording.summary.video.name = "no_such_video.mp4".to_owned();
    recording.write(folder.path()).unwrap();

    let missing = compare(&Case::setting(folder.path().to_path_buf()), &Folders::default()).unwrap_err();

    assert!(matches!(missing, Error::Fixture(_)));
}

#[test]
fn an_earlier_folder_without_the_frames_of_the_video_is_reported_before_the_video_is_even_looked_for() {
    let folder = TempDir::new().unwrap();
    let earlier = TempDir::new().unwrap();
    let mut recording = extract();
    recording.summary.video.name = "no_such_video.mp4".to_owned();
    recording.write(folder.path()).unwrap();
    let folders = Folders {
        frames: None,
        same_as: Some(earlier.path().to_path_buf()),
    };

    let error = compare(&Case::recorded(folder.path().to_path_buf()), &folders).unwrap_err();

    assert!(matches!(
        error,
        Error::Reference(reference::error::Error::Read { path, .. })
            if path == frames_path(earlier.path(), "no_such_video.mp4")
    ));
}

#[test]
fn results_come_back_in_the_order_of_the_items_even_when_the_last_finish_first() {
    let items: Vec<usize> = (0..6).collect();
    let finished = (Mutex::new(Vec::new()), Condvar::new());

    let results = each_in_order(&items, items.len(), |&item| {
        let (order, turn) = &finished;
        let mut order = turn
            .wait_while(order.lock().unwrap(), |order| order.len() + item + 1 < items.len())
            .unwrap();
        order.push(item);
        turn.notify_all();
        item * 10
    });

    assert_eq!(finished.0.into_inner().unwrap(), vec![5, 4, 3, 2, 1, 0]);
    assert_eq!(results, vec![0, 10, 20, 30, 40, 50]);
}

#[test]
fn every_case_is_compared_and_answered() {
    let folders: Vec<TempDir> = (0..3).map(|_| TempDir::new().unwrap()).collect();
    let cases: Vec<Case> = folders
        .iter()
        .map(|folder| Case::recorded(folder.path().to_path_buf()))
        .collect();

    let answers = compare_all(&cases, 2, &Folders::default());

    assert_eq!(answers.len(), 3);
    assert!(answers.iter().all(|answer| matches!(answer, Err(Error::Reference(_)))));
}

#[test]
fn nothing_to_compare_gives_nothing_back() {
    assert!(compare_all(&[], 6, &Folders::default()).is_empty());
}

#[test]
fn the_reference_compared_with_itself_does_not_differ_and_ignores_nothing() {
    let recording = extract();

    let comparison = compared(EXTRACT, &same_pass(&recording), &recording, 1.5).unwrap();

    assert!(!comparison.differs());
    assert_eq!(comparison.ignored, []);
    assert_eq!(comparison.decode_errors, DecodeErrors::default());
    assert_eq!(comparison.seconds, 1.5);
}

#[test]
fn the_detections_are_compared_at_the_frame_rate_of_the_reference() {
    let mut recording = extract();
    recording.summary.stream.avg_frame_rate = types::rational::Rational::new(0, 1).unwrap();

    let refused = compared(EXTRACT, &same_pass(&recording), &recording, 1.5).unwrap_err();

    assert!(matches!(refused, Error::Rate { .. }));
}

#[test]
fn a_frame_more_or_another_timestamp_or_key_frame_stops_the_comparison_rather_than_counting_gaps() {
    let recording = extract();
    let pass = same_pass(&recording);
    let mut fewer = FrameDetections::default();
    for _ in 1..pass.frames.entries().len() {
        fewer.push([]);
    }
    let shorter = Pass {
        frames: Frames::new(pass.frames.entries()[1..].to_vec(), fewer).unwrap(),
        reports: pass.reports.clone(),
    };
    let moved = with_entries(&pass, |entries| entries[40].pts = Some(Pts::new(1)));
    let keyed = with_entries(&pass, |entries| entries[41].key_frame = !entries[41].key_frame);

    let gaps: Vec<Option<InputGap>> = [moved, keyed, shorter]
        .iter()
        .map(|current| match compared(EXTRACT, current, &recording, 1.0) {
            Err(Error::Inputs { gap, .. }) => Some(gap),
            _ => None,
        })
        .collect();

    assert_eq!(
        gaps,
        [
            Some(InputGap::Entry {
                frame: FrameIndex::new(40),
                fields: vec![Field::Pts]
            }),
            Some(InputGap::Entry {
                frame: FrameIndex::new(41),
                fields: vec![Field::KeyFrame]
            }),
            Some(InputGap::FrameCount {
                previous: 1230,
                current: 1229
            })
        ]
    );
}

#[test]
fn frames_lost_at_an_edge_must_match_those_of_the_v020_but_the_v011_does_not_know_them() {
    let lost_after = |recording: &Recording| {
        let mut pass = same_pass(recording);
        pass.reports.edges = EdgeLosses {
            before_first: 0,
            after_last: 2,
        };
        pass
    };
    let v020 = extract();
    let v011 = Recording::read(&committed(Reference::V011, EXTRACT)).unwrap();

    let refused = compared(EXTRACT, &lost_after(&v020), &v020, 1.0).unwrap_err();
    let compared_with_v011 = compared(EXTRACT, &lost_after(&v011), &v011, 1.0).unwrap();

    assert!(matches!(
        refused,
        Error::Inputs {
            gap: InputGap::Losses { .. },
            ..
        }
    ));
    assert_eq!(compared_with_v011.ignored, [span(1215, 1229)]);
}

#[test]
fn a_decoding_error_of_one_side_is_counted_and_damages_both_sides_up_to_the_next_key_frame() {
    let recording = extract();
    let mut current = with_entries(&same_pass(&recording), |entries| entries[100].decode_error = true);
    current.reports.error_frames = vec![FrameIndex::new(100)];

    let comparison = compared(EXTRACT, &current, &recording, 1.0).unwrap();

    assert_eq!(
        comparison.decode_errors,
        DecodeErrors {
            reference: 0,
            current: 1,
            differing: 1
        }
    );
    assert_eq!(comparison.ignored.len(), 1);
    assert!(comparison.ignored[0].contains(FrameIndex::new(85)));
    assert!(comparison.ignored[0].contains(FrameIndex::new(100)));
}

#[test]
fn a_frame_in_error_only_in_the_reference_damages_the_tracks_and_detections_of_both_sides_in_either_version() {
    let in_error = |mut recording: Recording| {
        let first_point = recording.tracks[0].points()[0].frame;
        if let Some(decoder) = recording.summary.decoder.as_mut() {
            decoder.error_frames = vec![first_point];
        }
        if let Some(probe) = recording.summary.ffprobe.as_mut() {
            probe.error_frames = vec![first_point];
        }
        (recording, first_point)
    };

    let found: Vec<(bool, TrackCounts)> = [
        extract(),
        Recording::read(&committed(Reference::V011, EXTRACT)).unwrap(),
    ]
    .into_iter()
    .map(in_error)
    .map(|(recording, first_point)| {
        let comparison = compared(EXTRACT, &same_pass(&extract()), &recording, 1.0).unwrap();
        (
            comparison.ignored.iter().any(|span| span.contains(first_point)),
            comparison.tracks.counts(),
        )
    })
    .collect();

    let cut = TrackCounts {
        reference: 4,
        paired: 3,
        agreeing: 3,
        lost: 1,
        added: 1,
    };
    assert_eq!(found, [(true, cut), (true, cut)]);
}

#[test]
fn the_damaged_spans_of_the_reference_are_ignored_beside_those_drawn_again() {
    let mut recording = extract();
    recording.summary.damaged_spans = vec![span(10, 20), span(18, 30)];

    let comparison = compared(EXTRACT, &same_pass(&recording), &recording, 1.0).unwrap();

    assert_eq!(comparison.ignored, [span(10, 30)]);
}

#[test]
fn the_detection_settings_come_from_the_recording_and_one_the_engine_refuses_stops_before_any_reading() {
    let folder = TempDir::new().unwrap();
    let recorded = committed(Reference::V020, "settings/osd");
    for entry in std::fs::read_dir(&recorded).unwrap() {
        let path = entry.unwrap().path();
        std::fs::copy(&path, folder.path().join(path.file_name().unwrap())).unwrap();
    }
    let summary = folder.path().join("video.json");
    let text = std::fs::read_to_string(&summary).unwrap();
    let forged = text.replace("\"threshold\": 12.0,", "\"threshold\": -1.0,");
    assert_ne!(forged, text);
    std::fs::write(&summary, forged).unwrap();

    let error = compare(&Case::setting(folder.path().to_path_buf()), &Folders::default()).unwrap_err();

    assert!(matches!(error, Error::Settings { .. }), "{error:?}");
}
