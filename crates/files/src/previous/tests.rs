use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use jiff::civil::date;
use types::language::Language;

use super::{Kept, Reason, remove_previous_outputs};
use crate::aside::Stamp;
use crate::names::Names;
use crate::table::{Column, Headings};

const LEGACY_TABLE: &str = "id,start,end,start_s,duration_s,hits,chord_px,path_px,speed_px_s,max_area_px,peak_amplitude,filled_frames\r\n1,0:00.00,0:00.50,0.0,0.5,3,10.0,10.0,20.0,9,80.0,0\r\n";

#[derive(Debug)]
struct Debugged;

impl Headings for Debugged {
    fn heading(&self, column: Column, _language: Language) -> String {
        format!("{column:?}")
    }
}

fn folder_with(files: &[&str], folders: &[&str]) -> tempfile::TempDir {
    let dest = tempfile::tempdir().unwrap();
    for name in files {
        fs::write(dest.path().join(name), LEGACY_TABLE).unwrap();
    }
    for name in folders {
        fs::create_dir(dest.path().join(name)).unwrap();
        fs::write(dest.path().join(name).join("track_01_0m00s.mp4"), "").unwrap();
    }
    dest
}

fn half_past_two() -> Stamp {
    Stamp::at(date(2026, 10, 16).at(14, 30, 0, 0))
}

fn cleaned(dest: &Path) -> Vec<Kept> {
    remove_previous_outputs(
        dest,
        OsStr::new("v"),
        Names::new(Language::French),
        &Debugged,
        half_past_two(),
    )
    .unwrap()
}

fn left_in(dest: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dest)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn the_outputs_of_both_languages_and_of_the_old_names_are_removed_with_their_partial_files() {
    let outputs = [
        "v_pistes.csv",
        "v_tracks.csv",
        "v.tracks.csv",
        "v_resume.png",
        "v_summary_2_of_3.png",
        "v.tracks_000m-010m.png",
        "v_annotee.mp4",
        "v_annotated.mp4",
        "v_boxes.mp4",
        ".v_annotee.mp4.partial",
        ".v_summary.png.partial",
        ".v_tracks.csv.partial",
    ];
    let dest = folder_with(&outputs, &["extraits", "clips", "split"]);
    assert_eq!(cleaned(dest.path()), []);
    assert_eq!(left_in(dest.path()), Vec::<String>::new());
}

#[test]
fn the_files_of_the_user_of_other_videos_and_named_almost_like_ours_are_left() {
    let others = ["notes.txt", "w_pistes.csv", "v_summary_3_sur_8.png", "w_params.json"];
    let dest = folder_with(&others, &["autres"]);
    cleaned(dest.path());
    assert_eq!(
        left_in(dest.path()),
        [
            "autres",
            "notes.txt",
            "v_summary_3_sur_8.png",
            "w_params.json",
            "w_pistes.csv"
        ]
    );
}

#[test]
fn the_params_file_that_marks_a_finished_folder_is_removed_with_its_partial_file() {
    let dest = folder_with(&["params.json", ".params.json.partial", "notes.txt"], &[]);
    assert_eq!(cleaned(dest.path()), []);
    assert_eq!(left_in(dest.path()), ["notes.txt"]);
}

#[test]
fn a_table_with_verdicts_is_set_aside_under_a_dated_name_whatever_its_language_and_the_rest_is_removed() {
    let dest = folder_with(&["v_resume.png"], &[]);
    let annotated = LEGACY_TABLE
        .replacen("filled_frames", "filled_frames,verdict", 1)
        .replacen(",0\r\n", ",0,oui\r\n", 1);
    for name in ["v_pistes.csv", "v_tracks.csv"] {
        fs::write(dest.path().join(name), &annotated).unwrap();
    }
    assert_eq!(
        cleaned(dest.path()),
        [
            Kept::SetAside {
                path: dest.path().join("v_pistes.csv"),
                dated: dest.path().join("v_pistes_2026-10-16_1430.csv"),
                reason: Reason::Verdicts,
            },
            Kept::SetAside {
                path: dest.path().join("v_tracks.csv"),
                dated: dest.path().join("v_tracks_2026-10-16_1430.csv"),
                reason: Reason::Verdicts,
            },
        ]
    );
    assert_eq!(
        left_in(dest.path()),
        ["v_pistes_2026-10-16_1430.csv", "v_tracks_2026-10-16_1430.csv"]
    );
    assert_eq!(
        fs::read_to_string(dest.path().join("v_pistes_2026-10-16_1430.csv")).unwrap(),
        annotated
    );
}

#[test]
fn a_table_with_semicolons_an_extra_cell_latin_text_or_nothing_is_set_aside_as_of_an_unknown_form() {
    let semicolons = LEGACY_TABLE.replace(',', ";").into_bytes();
    let extra_cell = LEGACY_TABLE.replacen(",0\r\n", ",0,yes\r\n", 1).into_bytes();
    let mut latin = LEGACY_TABLE.as_bytes().to_vec();
    latin.push(0xE9);
    for content in [semicolons, extra_cell, latin, Vec::new()] {
        let dest = tempfile::tempdir().unwrap();
        fs::write(dest.path().join("v.tracks.csv"), &content).unwrap();
        assert_eq!(
            cleaned(dest.path()),
            [Kept::SetAside {
                path: dest.path().join("v.tracks.csv"),
                dated: dest.path().join("v.tracks_2026-10-16_1430.csv"),
                reason: Reason::UnknownForm,
            }],
            "{}",
            String::from_utf8_lossy(&content)
        );
        assert_eq!(
            fs::read(dest.path().join("v.tracks_2026-10-16_1430.csv")).unwrap(),
            content
        );
    }
}

#[test]
fn a_table_set_aside_is_left_alone_by_the_next_cleanup() {
    let dest = folder_with(&[], &[]);
    fs::write(dest.path().join("v_pistes.csv"), "piste;verdict\r\n1;oui\r\n").unwrap();
    cleaned(dest.path());
    fs::write(dest.path().join("v_pistes.csv"), LEGACY_TABLE).unwrap();
    assert_eq!(cleaned(dest.path()), []);
    assert_eq!(left_in(dest.path()), ["v_pistes_2026-10-16_1430.csv"]);
}

#[test]
fn a_table_of_ours_saved_with_unix_line_ends_is_still_ours() {
    let dest = tempfile::tempdir().unwrap();
    fs::write(dest.path().join("v_tracks.csv"), LEGACY_TABLE.replace("\r\n", "\n")).unwrap();
    assert_eq!(cleaned(dest.path()), []);
    assert_eq!(left_in(dest.path()), Vec::<String>::new());
}

#[test]
fn a_video_name_decomposed_by_the_file_system_still_matches_its_outputs() {
    let decomposed = "re\u{0301}sume\u{0301}";
    let composed = "r\u{e9}sum\u{e9}";
    let dest = folder_with(
        &[&format!("{decomposed}_pistes.csv"), &format!("{decomposed}_resume.png")],
        &[],
    );
    remove_previous_outputs(
        dest.path(),
        OsStr::new(composed),
        Names::new(Language::French),
        &Debugged,
        half_past_two(),
    )
    .unwrap();
    assert_eq!(left_in(dest.path()), Vec::<String>::new());
}

#[test]
fn a_missing_folder_has_nothing_to_remove() {
    let dest = tempfile::tempdir().unwrap();
    assert_eq!(cleaned(&dest.path().join("absent")), []);
}

#[test]
fn a_folder_named_like_a_table_is_left_where_it_is_and_the_rest_is_removed() {
    let dest = folder_with(&["v_resume.png"], &["clips"]);
    fs::create_dir(dest.path().join("v_pistes.csv")).unwrap();
    assert_eq!(
        cleaned(dest.path()),
        [Kept::Left {
            path: dest.path().join("v_pistes.csv"),
            ours: true,
        }]
    );
    assert_eq!(left_in(dest.path()), ["v_pistes.csv"]);
}

#[cfg(unix)]
#[test]
fn a_link_named_like_the_clips_folder_is_removed_without_touching_its_target() {
    let dest = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    fs::write(elsewhere.path().join("keep.mp4"), "").unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), dest.path().join("clips")).unwrap();
    cleaned(dest.path());
    assert_eq!(left_in(dest.path()), Vec::<String>::new());
    assert!(elsewhere.path().join("keep.mp4").is_file());
}

#[cfg(unix)]
#[test]
fn a_table_that_cannot_be_read_stops_the_cleanup_after_the_params_file_and_before_any_output() {
    use std::os::unix::fs::PermissionsExt;

    use crate::error::Error;
    let dest = folder_with(&["params.json", "v_pistes.csv", "v_resume.png"], &["clips"]);
    let table = dest.path().join("v_pistes.csv");
    fs::set_permissions(&table, fs::Permissions::from_mode(0o000)).unwrap();
    let error = remove_previous_outputs(
        dest.path(),
        OsStr::new("v"),
        Names::new(Language::French),
        &Debugged,
        half_past_two(),
    )
    .unwrap_err();
    fs::set_permissions(&table, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(&error, Error::Read { path, .. } if *path == table), "{error}");
    assert_eq!(left_in(dest.path()), ["clips", "v_pistes.csv", "v_resume.png"]);
}
