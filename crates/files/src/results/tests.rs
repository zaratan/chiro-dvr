use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use types::language::Language;

use super::{Found, read_results};
use crate::failures::{FailureRecord, record_failure};
use crate::table::{Column, Headings};

#[derive(Debug)]
struct Named;

impl Headings for Named {
    fn heading(&self, column: Column, _language: Language) -> String {
        format!("{column:?}")
    }
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    bytes.extend([8, 6, 0, 0, 0]);
    bytes
}

fn params(state: &str) -> String {
    format!(
        r#"{{
  "video": "/nuit/v.mp4",
  "RenderConfig": {{ "max_tracks": 300, "zoom": "auto" }},
  "version": "0.2.0",
  {state}
  "track_count": 2,
  "length_s": 40.4,
  "not_analysed_s": 0.0,
  "language": "fr",
  "date": "2026-10-16T10:18:03+02:00",
  "tracks": [
    {{ "id": 1, "start_s": 4.8, "duration_s": 9.62, "color": [255, 214, 10], "summary": "v_resume_1_sur_2.png", "clip": "extraits/piste_01_0m04s.mp4", "zoom": null }},
    {{ "id": 2, "start_s": 23.48, "duration_s": 0.57, "color": [76, 201, 240], "summary": "v_resume_2_sur_2.png", "clip": "extraits/piste_02_0m23s.mp4", "zoom": "extraits/piste_02_0m23s_zoom.mp4" }}
  ]
}}"#
    )
}

fn finished_folder(root: &Path, name: &str) {
    let dest = root.join(name);
    fs::create_dir_all(dest.join("extraits")).unwrap();
    fs::write(dest.join("params.json"), params(r#""state": "done","#)).unwrap();
    fs::write(dest.join("v_resume_1_sur_2.png"), png(1989, 1215)).unwrap();
    fs::write(dest.join("v_resume_2_sur_2.png"), png(1989, 1215)).unwrap();
    fs::write(
        dest.join("v_pistes.csv"),
        "\u{feff}Track;Verdict;StartSeconds\r\n1;oui;4,8\r\n2;;23,48\r\n",
    )
    .unwrap();
}

fn failure(video: &str) -> FailureRecord {
    FailureRecord {
        video: video.to_owned(),
        cause: "damaged".to_owned(),
        place: String::new(),
        detail: None,
        date: "2026-10-16T08:19:00Z".to_owned(),
    }
}

#[test]
fn a_finished_folder_brings_its_tracks_images_with_their_size_table_and_verdicts() {
    let root = tempfile::tempdir().unwrap();
    finished_folder(root.path(), "v");
    let folders = read_results(root.path(), &Named);
    assert_eq!(folders.len(), 1);
    let Found::Finished(finished) = &folders[0].found else {
        panic!("{:?}", folders[0].found);
    };
    assert_eq!(finished.written.tracks.len(), 2);
    assert_eq!(finished.written.render.max_tracks, 300);
    let images: Vec<(&str, u32)> = finished
        .summaries
        .iter()
        .map(|image| (image.file.as_str(), image.width))
        .collect();
    assert_eq!(images, [("v_resume_1_sur_2.png", 1989), ("v_resume_2_sur_2.png", 1989)]);
    assert_eq!(finished.table.as_deref(), Some("v_pistes.csv"));
    assert_eq!(finished.verdicts, [Some("oui".to_owned()), None]);
    assert_eq!(finished.annotated, None);
}

#[test]
fn a_folder_with_our_outputs_but_no_parameters_is_unfinished() {
    let root = tempfile::tempdir().unwrap();
    let dest = root.path().join("v");
    fs::create_dir_all(dest.join("extraits")).unwrap();
    fs::write(dest.join("extraits/piste_01_0m04s.mp4"), "").unwrap();
    fs::write(dest.join(".v_pistes.csv.partial"), "").unwrap();
    assert_eq!(read_results(root.path(), &Named)[0].found, Found::Unfinished);
}

#[test]
fn parameters_written_by_an_older_version_mean_a_video_done_by_it_not_an_interrupted_one() {
    let root = tempfile::tempdir().unwrap();
    let dest = root.path().join("v");
    fs::create_dir_all(&dest).unwrap();
    fs::write(dest.join("params.json"), params("")).unwrap();
    assert_eq!(read_results(root.path(), &Named)[0].found, Found::Older);
}

#[test]
fn a_folder_of_the_user_or_a_hidden_one_is_not_a_video() {
    let root = tempfile::tempdir().unwrap();
    let mine = root.path().join("photos");
    fs::create_dir_all(&mine).unwrap();
    fs::write(mine.join("params.json"), r#"{ "zoom": 2 }"#).unwrap();
    fs::write(mine.join("plage.jpg"), "").unwrap();
    fs::create_dir_all(root.path().join("Vacances/clips")).unwrap();
    fs::write(root.path().join("Vacances/clips/plongee.mp4"), "").unwrap();
    fs::create_dir_all(root.path().join(".cache/v")).unwrap();
    fs::write(root.path().join("resultats.html"), "").unwrap();
    assert_eq!(read_results(root.path(), &Named), []);
}

#[test]
fn a_failure_without_a_folder_stands_alone_and_one_beside_results_keeps_both() {
    let root = tempfile::tempdir().unwrap();
    finished_folder(root.path(), "b");
    record_failure(root.path(), OsStr::new("a"), &failure("a.mp4")).unwrap();
    record_failure(root.path(), OsStr::new("b"), &failure("b.mp4")).unwrap();
    let folders = read_results(root.path(), &Named);
    assert_eq!(folders.len(), 2);
    assert_eq!(folders[0].name, "a");
    assert_eq!(folders[0].found, Found::Absent);
    assert_eq!(folders[0].failure, Some(failure("a.mp4")));
    assert!(matches!(folders[1].found, Found::Finished(_)));
    assert_eq!(folders[1].failure, Some(failure("b.mp4")));
}

#[test]
fn an_image_listed_but_missing_on_the_disk_is_left_out() {
    let root = tempfile::tempdir().unwrap();
    finished_folder(root.path(), "v");
    fs::remove_file(root.path().join("v/v_resume_2_sur_2.png")).unwrap();
    let folders = read_results(root.path(), &Named);
    let Found::Finished(finished) = &folders[0].found else {
        panic!("{:?}", folders[0].found);
    };
    assert_eq!(finished.summaries.len(), 1);
}

#[test]
fn the_folders_come_in_the_order_of_their_names() {
    let root = tempfile::tempdir().unwrap();
    for name in ["video_097", "video_089", "video_092"] {
        finished_folder(root.path(), name);
    }
    let names: Vec<_> = read_results(root.path(), &Named)
        .into_iter()
        .map(|folder| folder.name)
        .collect();
    assert_eq!(names, ["video_089", "video_092", "video_097"]);
}
