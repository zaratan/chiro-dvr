use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};

use reference::location::committed;
use reference::recording::Recording;
use tempfile::TempDir;

use super::{FRAMES_FILE, compare, compare_all, compared, each_in_order, frames_path};
use crate::case::{Case, EXTRACT};
use crate::error::Error;

#[test]
fn the_frames_of_a_video_go_to_a_folder_named_after_it() {
    assert_eq!(
        frames_path(Path::new("out"), "video_092_original.mp4"),
        PathBuf::from("out/video_092_original").join(FRAMES_FILE)
    );
}

#[test]
fn a_recording_folder_without_a_summary_is_reported() {
    let folder = TempDir::new().unwrap();

    let error = compare(&Case::recorded(folder.path().to_path_buf()), None).unwrap_err();

    assert!(matches!(error, Error::Reference(_)));
}

#[test]
fn a_recording_whose_video_is_missing_is_reported_before_any_detection() {
    let folder = TempDir::new().unwrap();
    let mut recording = Recording::read(&committed(EXTRACT)).unwrap();
    recording.summary.video.name = "no_such_video.mp4".to_owned();
    recording.write(folder.path()).unwrap();

    let missing = compare(&Case::committed(folder.path().to_str().unwrap()), None).unwrap_err();

    assert!(matches!(missing, Error::Fixture(_)));
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

    let answers = compare_all(&cases, 2, None);

    assert_eq!(answers.len(), 3);
    assert!(answers.iter().all(|answer| matches!(answer, Err(Error::Reference(_)))));
}

#[test]
fn nothing_to_compare_gives_nothing_back() {
    assert!(compare_all(&[], 6, None).is_empty());
}

#[test]
fn the_detections_are_compared_at_the_frame_rate_of_the_reference() {
    let mut recording = Recording::read(&committed(EXTRACT)).unwrap();
    let comparison = compared(recording.frames.detections(), &recording, 1.5).unwrap();
    recording.summary.stream.avg_frame_rate = types::rational::Rational::new(0, 1).unwrap();

    let refused = compared(recording.frames.detections(), &recording, 1.5).unwrap_err();

    assert!(comparison.detections.holds());
    assert_eq!(comparison.seconds, 1.5);
    assert!(matches!(refused, Error::Rate { .. }));
}
