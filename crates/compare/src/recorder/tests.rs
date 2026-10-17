use std::fs;
use std::path::Path;

use reference::fixture::{fixture, fixtures};
use reference::location::Reference;
use reference::recording::{Recording, SUMMARY_FILE};
use reference::video::{DECODED_FORMAT, VideoSummary};
use tempfile::TempDir;

use super::{Places, holds, record, refuse_overlap};
use crate::case::Case;
use crate::comparison::frames_path;
use crate::error::Error;
use crate::testing::{SPOT, build, probed};

fn spot_case(folder: &Path) -> Case {
    let recording = folder.join("case");
    fs::create_dir_all(&recording).unwrap();
    probed(&fixture(SPOT).unwrap())
        .write(&recording.join(SUMMARY_FILE))
        .unwrap();
    Case::committed(Reference::V011, recording.to_str().unwrap())
}

fn into(out: &Path) -> Places<'_> {
    Places { out, same_as: None }
}

#[test]
fn a_case_is_recorded_into_its_folder_in_the_second_format_and_reads_back_with_its_tracks() {
    let (cases, out) = (TempDir::new().unwrap(), TempDir::new().unwrap());

    let recorded = record(&spot_case(cases.path()), &build(), &into(out.path())).unwrap();

    assert_eq!(recorded.directory, out.path().join("case"));
    let read = Recording::read(&recorded.directory).unwrap();
    assert_eq!(read.summary, recorded.summary);
    assert_eq!(read.summary.format_version, DECODED_FORMAT);
    assert_eq!(read.frames.entries().len(), 40);
    assert_eq!(read.tracks.len(), read.summary.counts.tracks);
    assert_eq!(read.rows, None);
    assert_eq!(recorded.same_as, None);
}

#[test]
fn a_case_recorded_again_is_the_same_as_the_frames_of_an_earlier_compare_pass_named_after_its_video() {
    let (cases, first, earlier, second) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let case = spot_case(cases.path());
    let recorded = record(&case, &build(), &into(first.path())).unwrap();
    let frames = frames_path(earlier.path(), &recorded.name);
    fs::create_dir_all(frames.parent().unwrap()).unwrap();
    Recording::read(&recorded.directory)
        .unwrap()
        .frames
        .write(&frames)
        .unwrap();

    let again = record(
        &case,
        &build(),
        &Places {
            out: second.path(),
            same_as: Some(earlier.path()),
        },
    )
    .unwrap();

    assert!(again.same_as.is_some() && again.holds());
    assert!(holds(&[Ok(again)]));
}

#[test]
fn a_video_whose_stream_is_not_the_recorded_one_is_refused_before_anything_is_written() {
    let (cases, out) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let case = spot_case(cases.path());
    let path = case.recording().join(SUMMARY_FILE);
    let mut summary = VideoSummary::read(&path).unwrap();
    summary.stream.height += 2;
    summary.write(&path).unwrap();

    let refused = record(&case, &build(), &into(out.path())).unwrap_err();

    assert!(matches!(refused, Error::Stream { .. }));
    assert_eq!(fs::read_dir(out.path()).unwrap().count(), 0);
}

#[test]
fn a_recording_written_into_or_around_a_reference_is_refused_before_any_detection() {
    let (reference, elsewhere) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let protected = [fixtures(), reference.path().to_path_buf()];

    let refused: Vec<bool> = [
        fixtures().join("reference"),
        reference.path().join("data"),
        reference.path().parent().unwrap().to_path_buf(),
    ]
    .iter()
    .map(|out| matches!(refuse_overlap(out, &protected), Err(Error::Overlap { .. })))
    .collect();

    assert_eq!(refused, [true, true, true]);
    assert!(refuse_overlap(&elsewhere.path().join("v020"), &protected).is_ok());
}

#[test]
fn a_recording_folder_is_judged_before_it_exists_and_is_never_created_by_the_judgement() {
    let (reference, elsewhere) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let protected = [reference.path().to_path_buf()];
    let around = elsewhere
        .path()
        .join("new")
        .join("..")
        .join("..")
        .join(reference.path().file_name().unwrap())
        .join("data");
    let accepted = elsewhere.path().join("new").join("v020");

    let refused = refuse_overlap(&around, &protected);

    assert!(matches!(refused, Err(Error::Overlap { .. })), "{refused:?}");
    assert!(refuse_overlap(&accepted, &protected).is_ok());
    assert!(!reference.path().join("data").exists());
    assert!(!elsewhere.path().join("new").exists());
}

#[cfg(unix)]
#[test]
fn a_recording_folder_reached_through_a_link_into_a_reference_is_refused_without_writing_there() {
    let (reference, elsewhere) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let link = elsewhere.path().join("link");
    std::os::unix::fs::symlink(reference.path(), &link).unwrap();

    let refused = refuse_overlap(&link.join("new").join("data"), &[reference.path().to_path_buf()]);

    assert!(matches!(refused, Err(Error::Overlap { .. })), "{refused:?}");
    assert!(!reference.path().join("new").exists());
}

#[test]
fn a_run_holds_only_with_at_least_one_case_and_no_failure() {
    let failed: Result<_, Error> = Err(Error::Stream {
        video: SPOT.to_owned(),
        expected: String::new(),
        found: String::new(),
    });

    assert!(!holds(&[]));
    assert!(!holds(&[failed]));
}
