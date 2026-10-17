use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use types::detection::{Detection, Strength};
use types::frame::FrameIndex;
use types::language::Language;
use types::track::Track;

use super::{
    Names, Output, Part, composed, is_hidden, is_partial_params, name_key, params_json, partial_file, previous_output,
};

fn track(id: usize) -> Track {
    let point = Detection {
        frame: FrameIndex::new(0),
        x: 1.0,
        y: 1.0,
        left: 0.0,
        top: 0.0,
        width: 3.0,
        height: 3.0,
        area: 9.0,
        amplitude: 80.0,
        strength: Strength::Strong,
    };
    Track::new(id, vec![point]).unwrap()
}

fn every_name(language: Language) -> Vec<PathBuf> {
    let (names, dest, stem) = (Names::new(language), Path::new("out/v"), OsStr::new("v"));
    let clips = names.clips_dir(dest);
    vec![
        clips.join(names.clip(&track(1), 0.0, 1)),
        clips.join(names.zoomed_clip(&track(1), 0.0, 1)),
        names.tracks_csv(dest, stem),
        names.summary_png(dest, stem, None, None),
        names.background_png(dest, stem),
        names.annotated_video(dest, stem),
    ]
}

#[test]
fn english_names_for_an_english_reader() {
    assert_eq!(
        every_name(Language::English),
        [
            "out/v/clips/track_01_0m00s.mp4",
            "out/v/clips/track_01_0m00s_zoom.mp4",
            "out/v/v_tracks.csv",
            "out/v/v_summary.png",
            "out/v/v_background.png",
            "out/v/v_annotated.mp4",
        ]
        .map(PathBuf::from)
    );
}

#[test]
fn french_names_without_any_accent_that_apfs_would_decompose() {
    let names = every_name(Language::French);
    assert_eq!(
        names,
        [
            "out/v/extraits/piste_01_0m00s.mp4",
            "out/v/extraits/piste_01_0m00s_zoom.mp4",
            "out/v/v_pistes.csv",
            "out/v/v_resume.png",
            "out/v/v_fond.png",
            "out/v/v_annotee.mp4",
        ]
        .map(PathBuf::from)
    );
    assert!(names.iter().all(|name| name.to_string_lossy().is_ascii()));
}

#[test]
fn params_json_keeps_its_english_name_in_french() {
    assert_eq!(params_json(Path::new("out/v")), PathBuf::from("out/v/params.json"));
}

#[test]
fn clip_time_drops_hundredths_and_the_track_number_tells_apart_two_tracks_of_the_same_second() {
    let names = Names::new(Language::English);
    assert_eq!(
        [names.clip(&track(7), 237.4, 8), names.clip(&track(8), 237.97, 8)],
        ["track_07_3m57s.mp4", "track_08_3m57s.mp4"]
    );
}

#[test]
fn clip_time_counts_minutes_past_the_hour() {
    assert_eq!(
        Names::new(Language::English).clip(&track(12), 3725.0, 12),
        "track_12_62m05s.mp4"
    );
}

#[test]
fn track_number_takes_the_width_of_the_track_count_so_the_finder_sorts_clips_in_numeric_order() {
    let names = Names::new(Language::English);
    for (track_count, expected) in [
        (99, "track_07_0m00s.mp4"),
        (100, "track_007_0m00s.mp4"),
        (1000, "track_0007_0m00s.mp4"),
    ] {
        assert_eq!(names.clip(&track(7), 0.0, track_count), expected);
    }
}

#[test]
fn french_and_zoomed_clips_follow_the_same_number_width() {
    let names = Names::new(Language::French);
    assert_eq!(
        [names.clip(&track(7), 0.0, 100), names.zoomed_clip(&track(7), 0.0, 100)],
        ["piste_007_0m00s.mp4", "piste_007_0m00s_zoom.mp4"]
    );
}

#[test]
fn summary_of_a_long_video_carries_its_period_in_minutes() {
    let summary = Names::new(Language::English).summary_png(Path::new("out/v"), OsStr::new("v"), Some(0..10), None);
    assert_eq!(summary, PathBuf::from("out/v/v_summary_000m-010m.png"));
}

#[test]
fn a_time_before_the_start_of_the_video_is_named_as_zero() {
    assert_eq!(
        Names::new(Language::English).clip(&track(1), -0.4, 1),
        "track_01_0m00s.mp4"
    );
}

#[test]
fn an_accented_video_name_is_kept_as_given() {
    let csv = Names::new(Language::French).tracks_csv(Path::new("out"), OsStr::new("nuit_été"));
    assert_eq!(csv, PathBuf::from("out/nuit_été_pistes.csv"));
}

#[test]
fn a_summary_cut_into_parts_numbers_each_part_padded_to_sort_in_order() {
    let part = |number| Some(Part { number, count: 12 });
    let (dest, stem) = (Path::new("out/v"), OsStr::new("v"));
    assert_eq!(
        Names::new(Language::French).summary_png(dest, stem, None, part(2)),
        PathBuf::from("out/v/v_resume_02_sur_12.png")
    );
    assert_eq!(
        Names::new(Language::English).summary_png(dest, stem, Some(10..16), part(12)),
        PathBuf::from("out/v/v_summary_010m-016m_12_of_12.png")
    );
}

fn recognised(name: &str) -> Option<Output> {
    previous_output(name.as_bytes(), b"v").map(|(_, output)| output)
}

fn partial_name_of(name: &OsStr) -> String {
    let folder = tempfile::tempdir().unwrap();
    let partial = partial_file(&folder.path().join(name)).unwrap();
    partial.file_name().unwrap().to_str().unwrap().to_owned()
}

#[test]
fn every_name_we_write_is_recognised_as_a_previous_output_and_so_is_its_partial_file() {
    let (dest, stem) = (Path::new("out/v"), OsStr::new("v"));
    for language in [Language::French, Language::English] {
        let names = Names::new(language);
        let rendered = [
            names.annotated_video(dest, stem),
            names.background_png(dest, stem),
            names.summary_png(dest, stem, None, None),
            names.summary_png(dest, stem, Some(0..10), None),
            names.summary_png(dest, stem, None, Some(Part { number: 3, count: 8 })),
            names.summary_png(dest, stem, Some(10..16), Some(Part { number: 2, count: 12 })),
        ];
        let table = names.tracks_csv(dest, stem);
        let written = rendered
            .iter()
            .map(|path| (path, Output::Rendered))
            .chain([(&table, Output::TracksTable)]);
        for (path, output) in written {
            let name = path.file_name().unwrap().to_str().unwrap();
            assert_eq!(recognised(name), Some(output), "{name}");
            let partial = partial_name_of(path.file_name().unwrap());
            assert_eq!(recognised(&partial), Some(Output::Partial), "{partial}");
        }
        let clips = names.clips_dir(dest);
        assert_eq!(
            recognised(clips.file_name().unwrap().to_str().unwrap()),
            Some(Output::ClipsFolder)
        );
    }
}

#[test]
fn the_old_names_of_the_first_version_are_recognised() {
    assert_eq!(recognised("split"), Some(Output::ClipsFolder));
    assert_eq!(recognised("v.tracks.csv"), Some(Output::TracksTable));
    for name in ["v.tracks.png", "v.tracks_000m-010m.png", "v_boxes.mp4"] {
        assert_eq!(recognised(name), Some(Output::Rendered), "{name}");
    }
}

#[test]
fn the_summaries_of_every_period_and_every_part_are_recognised() {
    for name in [
        "v_resume_000m-010m.png",
        "v_summary_010m-016m.png",
        "v_resume_3_sur_8.png",
        "v_summary_03_of_12.png",
        "v_resume_000m-010m_1_sur_2.png",
        "v_summary_1200m-1210m.png",
    ] {
        assert_eq!(recognised(name), Some(Output::Rendered), "{name}");
    }
}

#[test]
fn names_that_only_look_like_outputs_are_not_recognised() {
    for name in [
        "params.json",
        "notes.txt",
        "autres",
        "w_pistes.csv",
        "v_pistes.csv.bak",
        "v_resume_1_of_2.png",
        "v_summary_3_sur_8.png",
        "v.tracks_1_of_2.png",
        "v.tracks_1__2.png",
        "v_resume_00m-10m.png",
        "v_resume_.png",
        "v_resume_3_sur_.png",
        "v_resume.jpg",
        ".v_pistes.csv.bak.partial",
        ".v_pistes.csv.Ab12C.partial",
        ".v_pistes.csv.Ab-12C.partial",
        ".v_pistes.csv.Ab12Cd7.partial",
        ".extraits.partial",
        "v_annotee.mp4.partial",
        "piste_01_0m00s.mp4",
        "v.png",
        "v_fond.jpg",
        "v_fond_1_sur_2.png",
    ] {
        assert_eq!(recognised(name), None, "{name}");
    }
}

#[test]
fn a_partial_file_is_created_empty_and_hidden_beside_its_final_name_with_a_unique_part() {
    let folder = tempfile::tempdir().unwrap();
    let partial = partial_file(&folder.path().join("v_pistes.csv")).unwrap();
    assert_eq!(partial.parent(), Some(folder.path()));
    assert_eq!(fs::read(&partial).unwrap(), b"");
    let name = partial.file_name().unwrap().to_str().unwrap();
    let unique = name
        .strip_prefix(".v_pistes.csv.")
        .and_then(|rest| rest.strip_suffix(".partial"))
        .unwrap();
    assert_eq!(unique.len(), 6, "{name}");
}

#[test]
fn two_partial_files_of_the_same_file_have_two_names_so_two_writers_never_share_one() {
    let folder = tempfile::tempdir().unwrap();
    let page = folder.path().join("resultats.html");
    let (first, second) = (partial_file(&page).unwrap(), partial_file(&page).unwrap());
    assert_ne!(first.to_path_buf(), second.to_path_buf());
    assert!(first.exists() && second.exists());
}

#[test]
fn a_path_without_a_file_name_has_no_partial_file() {
    let error = partial_file(Path::new("out/..")).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn a_hidden_partial_name_with_any_word_of_six_letters_or_digits_is_taken_for_ours() {
    assert_eq!(recognised(".v_pistes.csv.backup.partial"), Some(Output::Partial));
}

#[test]
fn a_partial_file_left_under_the_fixed_name_of_version_0_2_0_is_still_recognised() {
    for name in [
        ".v_pistes.csv.partial",
        ".v_annotee.mp4.partial",
        ".v.tracks.csv.partial",
    ] {
        assert_eq!(recognised(name), Some(Output::Partial), "{name}");
    }
}

#[test]
fn the_partial_files_of_the_params_file_are_recognised_under_both_names_and_nothing_else() {
    let unique = partial_name_of(OsStr::new("params.json"));
    for name in [unique.as_str(), ".params.json.partial"] {
        assert!(is_partial_params(name.as_bytes()), "{name}");
    }
    for name in ["params.json", ".params.json.bak.partial", ".v_pistes.csv.partial"] {
        assert!(!is_partial_params(name.as_bytes()), "{name}");
    }
}

#[test]
fn the_key_of_a_name_is_its_composed_form_so_both_writings_of_an_accent_meet() {
    assert_eq!(
        name_key(OsStr::new("nuit_e\u{301}te\u{301}")),
        "nuit_\u{e9}t\u{e9}".as_bytes()
    );
    assert_eq!(
        name_key(OsStr::new("nuit_\u{e9}t\u{e9}")),
        "nuit_\u{e9}t\u{e9}".as_bytes()
    );
}

#[cfg(unix)]
#[test]
fn the_key_of_a_name_that_is_not_utf8_keeps_its_bytes_so_two_such_names_stay_apart() {
    use std::os::unix::ffi::OsStrExt;
    assert_eq!(name_key(OsStr::from_bytes(b"vol_\xe9")), b"vol_\xe9");
    assert_ne!(
        name_key(OsStr::from_bytes(b"vol_\xe9")),
        name_key(OsStr::from_bytes(b"vol_\xe8"))
    );
}

#[test]
fn a_name_decomposed_by_the_file_system_is_composed_again() {
    assert_eq!(composed("nuit_e\u{301}te\u{301}"), "nuit_\u{e9}t\u{e9}");
}

#[test]
fn the_results_page_is_named_in_the_language_of_the_run() {
    assert_eq!(super::results_page(Language::French), "resultats.html");
    assert_eq!(super::results_page(Language::English), "results.html");
}

#[test]
fn a_dated_name_escapes_the_cleanup_so_a_table_set_aside_is_never_removed_nor_set_aside_again() {
    for dated in [
        "v_pistes_2026-10-16_1430.csv",
        "v_tracks_2026-10-16_1430_2.csv",
        "v.tracks_2026-10-16_1430.csv",
        ".v_pistes_2026-10-16_1430.csv.partial",
    ] {
        assert_eq!(previous_output(dated.as_bytes(), b"v"), None, "{dated}");
    }
}

#[test]
fn a_name_is_hidden_when_it_starts_with_a_dot_like_the_appledouble_copies_and_our_partial_files() {
    let partial = partial_name_of(OsStr::new("v_pistes.csv"));
    for hidden in [OsStr::new("._v.mp4"), OsStr::new(".DS_Store"), OsStr::new(&partial)] {
        assert!(is_hidden(hidden), "{hidden:?}");
    }
    for shown in ["v.mp4", "nuit.12", ""] {
        assert!(!is_hidden(OsStr::new(shown)), "{shown:?}");
    }
}
