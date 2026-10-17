use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use files::error::Error;
use files::failures::{FailureRecord, record_failure};
use files::refusals::{RefusalRecord, RefusedInput, record_refusals};
use files::table::{Column, Headings};
use page::labels::{Label, Labels};
use types::language::Language;

use super::{Wording, page_to_open, roots, write_page};
use crate::jobs::Job;

#[derive(Debug)]
struct Named;

impl Labels for Named {
    fn label(&self, label: &Label<'_>) -> String {
        format!("{label:?}")
    }
}

impl Headings for Named {
    fn heading(&self, column: Column, _language: Language) -> String {
        format!("{column:?}")
    }
}

const FRENCH: Wording<'static> = Wording {
    labels: &Named,
    headings: &Named,
    language: Language::French,
};

fn params(state: &str, tracks: usize) -> String {
    let entries: Vec<String> = (1..=tracks)
        .map(|id| {
            format!(
                r#"{{ "id": {id}, "start_s": {id}.5, "duration_s": 0.6, "color": [255, 214, 10], "summary": "v_resume.png", "clip": "extraits/piste_0{id}_0m0{id}s.mp4", "zoom": null }}"#
            )
        })
        .collect();
    format!(
        r#"{{ "video": "/nuit/v.mp4", "RenderConfig": {{ "max_tracks": 2 }}, "version": "0.2.0", "state": "{state}",
  "track_count": {tracks}, "length_s": 40.4, "not_analysed_s": 0.0, "language": "fr", "date": "2026-10-09T23:12:00+02:00",
  "tracks": [{}] }}"#,
        entries.join(",")
    )
}

fn finished(root: &Path, name: &str, state: &str, tracks: usize) {
    let dest = root.join(name);
    fs::create_dir_all(&dest).unwrap();
    fs::write(dest.join("params.json"), params(state, tracks)).unwrap();
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    png.extend(1879_u32.to_be_bytes());
    png.extend(1215_u32.to_be_bytes());
    fs::write(dest.join("v_resume.png"), png).unwrap();
}

fn job(root: &Path, name: &str) -> Job {
    Job {
        video: PathBuf::from(format!("/nuit/{name}.mp4")),
        dest: root.join(name),
    }
}

fn page(root: &Path) -> String {
    fs::read_to_string(root.join("resultats.html")).unwrap()
}

#[test]
fn the_page_lists_the_finished_videos_and_opens_only_those_of_this_run() {
    let root = tempfile::tempdir().unwrap();
    finished(root.path(), "video_092", "done", 2);
    finished(root.path(), "video_093", "done", 1);
    write_page(root.path(), &[job(root.path(), "video_093")], 1, FRENCH).unwrap();
    let html = page(root.path());
    assert!(html.contains(r#"<details class="video" id="v1">"#));
    assert!(html.contains(r#"<details class="video" id="v2" open>"#));
    assert!(html.contains("Tracks(3)"));
    assert!(html.contains(r#"data-src="video_092/extraits/piste_01_0m01s.mp4""#));
}

#[test]
fn a_video_not_yet_processed_waits_and_shows_its_results_once_done() {
    let root = tempfile::tempdir().unwrap();
    let jobs = [job(root.path(), "video_092")];
    write_page(root.path(), &jobs, 0, FRENCH).unwrap();
    assert!(page(root.path()).contains("WaitingBadge"));
    finished(root.path(), "video_092", "done", 2);
    write_page(root.path(), &jobs, 1, FRENCH).unwrap();
    assert!(!page(root.path()).contains("WaitingBadge"));
}

#[test]
fn a_failure_after_a_success_shows_the_failure_with_the_earlier_results() {
    let root = tempfile::tempdir().unwrap();
    finished(root.path(), "video_093", "done", 1);
    let record = FailureRecord {
        video: "video_093.mp4".to_owned(),
        cause: "damaged".to_owned(),
        place: "/nuit/batdetect/video_093".to_owned(),
        detail: Some("cannot decode".to_owned()),
        date: "2026-10-16T08:19:00Z".to_owned(),
    };
    record_failure(root.path(), OsStr::new("video_093"), &record).unwrap();
    write_page(root.path(), &[], 0, FRENCH).unwrap();
    let html = page(root.path());
    assert!(html.contains(r#"<section class="video failed" id="v1""#));
    assert!(html.contains(r##"href="#v1-previous""##));
}

#[test]
fn a_video_with_too_many_tracks_carries_the_limit_it_went_past() {
    let root = tempfile::tempdir().unwrap();
    finished(root.path(), "video_097", "too_many_tracks", 3);
    write_page(root.path(), &[], 0, FRENCH).unwrap();
    assert!(page(root.path()).contains("TooMany { video: &quot;v&quot;, count: 3, limit: 2 }"));
}

#[test]
fn a_page_of_the_same_name_written_by_someone_else_is_never_replaced() {
    let root = tempfile::tempdir().unwrap();
    finished(root.path(), "video_092", "done", 1);
    fs::write(root.path().join("resultats.html"), "<p>mes notes</p>").unwrap();
    let refused = write_page(root.path(), &[], 0, FRENCH);
    assert!(matches!(refused, Err(Error::Taken { .. })));
    assert_eq!(page(root.path()), "<p>mes notes</p>");
}

#[test]
fn our_page_in_the_other_language_goes_and_someone_else_s_stays() {
    let root = tempfile::tempdir().unwrap();
    finished(root.path(), "video_092", "done", 1);
    write_page(
        root.path(),
        &[],
        0,
        Wording {
            language: Language::English,
            ..FRENCH
        },
    )
    .unwrap();
    assert!(root.path().join("results.html").exists());
    write_page(root.path(), &[], 0, FRENCH).unwrap();
    assert!(!root.path().join("results.html").exists());
    fs::write(root.path().join("results.html"), "<p>notes</p>").unwrap();
    write_page(root.path(), &[], 0, FRENCH).unwrap();
    assert!(root.path().join("results.html").exists());
}

#[test]
fn no_results_folder_is_created_for_a_page() {
    let videos = tempfile::tempdir().unwrap();
    let root = videos.path().join("batdetect");
    write_page(&root, &[job(&root, "video_092")], 0, FRENCH).unwrap();
    assert!(!root.exists());
}

#[test]
fn the_title_is_the_evening_folder_beside_the_videos_or_the_chosen_folder() {
    let videos = tempfile::tempdir().unwrap();
    let beside = videos.path().join("nuit-12-oct/batdetect");
    let chosen = videos.path().join("resultats-octobre");
    for root in [&beside, &chosen] {
        fs::create_dir_all(root).unwrap();
        write_page(root, &[], 0, FRENCH).unwrap();
    }
    assert!(page(&beside).contains("<title>Heading — nuit-12-oct</title>"));
    assert!(page(&chosen).contains("<title>Heading — resultats-octobre</title>"));
}

#[test]
fn each_results_folder_of_a_batch_is_listed_once_in_the_order_met() {
    let a = Path::new("/nuit-a/batdetect");
    let b = Path::new("/nuit-b/batdetect");
    let jobs = [job(a, "v1"), job(b, "v2"), job(a, "v3")];
    assert_eq!(roots(&jobs), [a.to_path_buf(), b.to_path_buf()]);
}

#[test]
fn the_page_gets_the_same_rights_as_any_file_written_beside_it() {
    let root = tempfile::tempdir().unwrap();
    write_page(root.path(), &[], 0, FRENCH).unwrap();
    fs::write(root.path().join("reference.txt"), "").unwrap();
    assert_eq!(
        fs::metadata(root.path().join("resultats.html")).unwrap().permissions(),
        fs::metadata(root.path().join("reference.txt")).unwrap().permissions()
    );
}

#[test]
fn the_technical_detail_shows_only_when_the_advice_refers_to_it() {
    let root = tempfile::tempdir().unwrap();
    for (folder, cause) in [("video_093", "not-a-video"), ("video_094", "unexpected")] {
        let record = FailureRecord {
            video: format!("{folder}.mp4"),
            cause: cause.to_owned(),
            place: String::new(),
            detail: Some(format!("chain of {folder}")),
            date: "2026-10-16T08:19:00Z".to_owned(),
        };
        record_failure(root.path(), OsStr::new(folder), &record).unwrap();
    }
    write_page(root.path(), &[], 0, FRENCH).unwrap();
    let html = page(root.path());
    assert!(!html.contains("chain of video_093"));
    assert!(html.contains("chain of video_094"));
}

#[test]
fn the_page_to_open_at_the_end_is_ours_in_the_language_of_the_run_or_none() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(page_to_open(root.path(), Language::French), None);
    write_page(root.path(), &[], 0, FRENCH).unwrap();
    assert_eq!(
        page_to_open(root.path(), Language::French),
        Some(root.path().join("resultats.html"))
    );
    assert_eq!(page_to_open(root.path(), Language::English), None);
    fs::write(root.path().join("resultats.html"), "<p>mes notes</p>").unwrap();
    assert_eq!(page_to_open(root.path(), Language::French), None);
}

#[test]
fn a_relative_results_folder_is_titled_by_the_folder_it_stands_in() {
    let here = std::env::current_dir().unwrap();
    let evening = here.file_name().unwrap().to_string_lossy().into_owned();
    assert_eq!(super::title(Path::new("batdetect")), evening);
    assert_eq!(super::title(Path::new(".")), evening);
}

#[cfg(target_os = "linux")]
#[test]
fn two_folders_whose_names_differ_only_by_bytes_that_are_not_utf8_are_two_videos_on_the_page() {
    use std::os::unix::ffi::OsStrExt;
    let root = tempfile::tempdir().unwrap();
    for name in [b"vol_\xe9".as_slice(), b"vol_\xe8"] {
        let dest = root.path().join(OsStr::from_bytes(name));
        fs::create_dir_all(&dest).unwrap();
        fs::write(dest.join("params.json"), params("done", 1)).unwrap();
    }
    write_page(root.path(), &[], 0, FRENCH).unwrap();
    assert_eq!(page(root.path()).matches("<details class=\"video\"").count(), 2);
}

#[test]
fn a_folder_named_by_a_decomposed_accent_is_the_same_video_as_its_composed_job() {
    let root = tempfile::tempdir().unwrap();
    finished(root.path(), "nuit_e\u{301}te\u{301}", "done", 1);
    write_page(root.path(), &[job(root.path(), "nuit_\u{e9}t\u{e9}")], 1, FRENCH).unwrap();
    let html = page(root.path());
    assert_eq!(html.matches("<details class=\"video\"").count(), 1);
    assert!(html.contains(r#"<details class="video" id="v1" open>"#));
}

fn refused_input(path: &str, cause: &str) -> RefusedInput {
    RefusedInput {
        path: path.to_owned(),
        reason: "unopened".to_owned(),
        cause: Some(cause.to_owned()),
        todo: cause.to_owned(),
        videos: vec!["/nuit/a.mp4".to_owned(), "/nuit/A.mp4".to_owned()],
        detail: Some(format!("chain of {path}")),
    }
}

#[test]
fn the_refusals_of_the_launch_show_on_the_page_of_their_folder() {
    let root = tempfile::tempdir().unwrap();
    finished(root.path(), "video_092", "done", 1);
    let record = RefusalRecord {
        date: "2026-10-16T19:02:00Z".to_owned(),
        inputs: vec![refused_input("/nuit/video_093.mp4", "damaged")],
    };
    record_refusals(root.path(), &record).unwrap();
    write_page(root.path(), &[job(root.path(), "video_092")], 1, FRENCH).unwrap();
    let html = page(root.path());
    assert!(
        html.contains("RefusedInput { path: &quot;/<wbr>nuit/<wbr>video_093.mp4&quot;, reason: &quot;unopened&quot;")
    );
    assert!(html.contains("videos: &quot;/<wbr>nuit/<wbr>a.mp4, /<wbr>nuit/<wbr>A.mp4&quot;"));
    assert!(html.contains("RefusedWhen(Date { year: 2026"));
    assert!(html.contains("RefusedCount(1)"));
}

#[test]
fn the_recorded_detail_of_a_refused_video_shows_on_the_page() {
    let root = tempfile::tempdir().unwrap();
    let record = RefusalRecord {
        date: "2026-10-16T19:02:00Z".to_owned(),
        inputs: vec![refused_input("/nuit/video_094.mp4", "unsupported")],
    };
    record_refusals(root.path(), &record).unwrap();
    write_page(root.path(), &[], 0, FRENCH).unwrap();
    assert!(page(root.path()).contains("chain of /nuit/video_094.mp4"));
}

#[test]
fn a_folder_without_refusals_shows_no_refused_card() {
    let root = tempfile::tempdir().unwrap();
    finished(root.path(), "video_092", "done", 1);
    write_page(root.path(), &[job(root.path(), "video_092")], 1, FRENCH).unwrap();
    assert!(!page(root.path()).contains("RefusedCount"));
}
