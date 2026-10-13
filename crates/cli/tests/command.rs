#[path = "support/clips.rs"]
mod clips;
#[path = "support/flight.rs"]
mod flight;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::Value;

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect-rs");

#[cfg(test)]
fn run(arguments: &[&Path], lang: &str) -> Output {
    Command::new(BINARY)
        .args(arguments)
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", lang)
        .output()
        .unwrap()
}

#[cfg(test)]
fn flight() -> &'static Path {
    static FLIGHT: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    let (_folder, path) = FLIGHT.get_or_init(|| {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("flight.mp4");
        flight::write_flight(&path);
        (folder, path)
    });
    path
}

#[cfg(test)]
fn videos_with_a_flight(names: &[&str]) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let videos = root.path().join("in");
    fs::create_dir(&videos).unwrap();
    for name in names {
        fs::copy(flight(), videos.join(name)).unwrap();
    }
    (root, videos)
}

#[cfg(test)]
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[test]
fn a_video_gives_its_tracks_table_and_params_in_a_folder_named_after_it() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let table = fs::read_to_string(out.join("flight").join("flight_pistes.csv")).unwrap();
    assert_eq!(table.lines().count(), 2, "{table}");
    assert!(table.starts_with("id,start,end,start_s,duration_s,hits,"));
    assert!(out.join("flight").join("params.json").is_file());
    assert!(text(&output.stdout).starts_with("flight.mp4 : 1 piste -> "));
}

#[test]
fn outside_a_french_locale_the_table_is_named_in_english() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let output = run(&[&videos.join("flight.mp4"), Path::new("-o"), &out], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(out.join("flight").join("flight_tracks.csv").is_file());
    assert!(text(&output.stdout).starts_with("flight.mp4: 1 track -> "));
}

#[test]
fn a_video_gives_a_summary_image_taller_than_the_video_named_in_the_language_of_the_messages() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    for (lang, name) in [("fr_FR.UTF-8", "flight_resume.png"), ("C", "flight_summary.png")] {
        let output = run(&[&videos, Path::new("-o"), &out], lang);
        assert!(output.status.success(), "{}", text(&output.stderr));
        let image = fs::read(out.join("flight").join(name)).unwrap();
        assert_eq!(&image[..8], b"\x89PNG\r\n\x1a\n");
        let width = u32::from_be_bytes(image[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(image[20..24].try_into().unwrap());
        assert!(width >= 1440, "{width}");
        assert!(height > 1080, "{height}");
    }
}

#[test]
fn params_json_records_the_settings_by_configuration_and_the_threads_really_used() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    assert!(run(&[&videos, Path::new("-o"), &out], "C").status.success());
    let params: Value =
        serde_json::from_str(&fs::read_to_string(out.join("flight").join("params.json")).unwrap()).unwrap();
    for key in [
        "video",
        "fps",
        "DetectConfig",
        "TrackConfig",
        "StabilityConfig",
        "threads",
        "ignored_s",
        "damaged_s",
    ] {
        assert!(params.get(key).is_some(), "{key} missing");
    }
    assert!(params.get("mode").is_none() && params.get("ffprobe").is_none());
    assert_eq!(params["RenderConfig"]["box_pad"], 12);
    assert_eq!(params["RenderConfig"]["max_tracks"], 300);
    let encoders = params["RenderConfig"]["encoders"].as_array().unwrap();
    assert_eq!(encoders.last().unwrap(), "openh264");
    assert!(params["threads"]["decoding"].as_u64().unwrap() >= 1);
    assert!(params["threads"]["detection"].as_u64().unwrap() >= 1);
    assert_eq!(params["version"], env!("CARGO_PKG_VERSION"));
    assert_ne!(params["ffmpeg"].as_str().unwrap(), "");
    assert!((params["fps"].as_f64().unwrap() - 10.0).abs() < 1e-9);
    assert_eq!(params["damaged_frames"], 0);
}

#[test]
fn an_unreadable_video_fails_without_stopping_the_others_and_is_named() {
    let (root, videos) = videos_with_a_flight(&["ok.mp4"]);
    fs::write(videos.join("broken.mp4"), "not a video").unwrap();
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    assert_eq!(output.status.code(), Some(1));
    assert!(out.join("ok").join("ok_tracks.csv").is_file());
    assert!(!out.join("broken").exists());
    let errors = text(&output.stderr);
    let failures: Vec<&str> = errors.lines().filter(|line| line.contains(".mp4: ")).collect();
    assert_eq!(failures.len(), 1, "{errors}");
    assert!(failures[0].contains("broken.mp4: the detection failed: "), "{errors}");
}

#[test]
fn without_an_output_folder_the_command_refuses_to_start() {
    let (_root, videos) = videos_with_a_flight(&[]);
    let output = run(&[&videos], "C");
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("--output <FOLDER>"));
}

#[test]
fn a_folder_without_any_video_is_refused() {
    let (root, videos) = videos_with_a_flight(&[]);
    let output = run(&[&videos, Path::new("-o"), &root.path().join("out")], "C");
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("no video found"));
}

#[test]
fn two_videos_that_would_share_an_output_folder_are_refused_before_any_work() {
    let root = tempfile::tempdir().unwrap();
    let (first, second) = (root.path().join("a/night"), root.path().join("b/night"));
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    fs::copy(flight(), first.join("x.mp4")).unwrap();
    fs::copy(flight(), second.join("x.mp4")).unwrap();
    let out = root.path().join("out");
    let output = run(
        &[&first.join("x.mp4"), &second.join("x.mp4"), Path::new("-o"), &out],
        "C",
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("several videos would write to"));
    assert!(!out.exists());
}

#[test]
fn a_video_that_does_not_exist_fails_alone_and_is_named() {
    let (root, videos) = videos_with_a_flight(&["ok.mp4"]);
    let out = root.path().join("out");
    let missing = videos.join("absent.mp4");
    let output = run(&[&missing, &videos.join("ok.mp4"), Path::new("-o"), &out], "C");
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("absent.mp4: "));
    assert!(out.join("ok").join("ok_tracks.csv").is_file());
}

#[test]
fn an_output_that_is_a_file_makes_each_video_fail_after_its_detection() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let file = root.path().join("out");
    fs::write(&file, "").unwrap();
    let output = run(&[&videos, Path::new("-o"), &file], "C");
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("cannot create"));
}

#[cfg(target_os = "linux")]
#[test]
fn an_output_folder_whose_name_is_not_utf8_is_written_as_given() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join(OsStr::from_bytes(b"sortie_\xe9"));
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(out.join("flight").join("flight_tracks.csv").is_file());
    assert!(out.join("flight").join("clips").join("track_01_0m00s.mp4").is_file());
}

#[test]
fn each_track_gets_a_clip_named_after_its_start_that_holds_its_window() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let lengths = clips::lengths(&out.join("flight").join("extraits"));
    assert_eq!(lengths, [("piste_01_0m00s.mp4".to_owned(), 20)]);
}

#[test]
fn the_clips_of_an_earlier_run_are_removed_before_the_new_ones_are_written() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let folder = out.join("flight").join("clips");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("track_07_1m00s.mp4"), "").unwrap();
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(clips::lengths(&folder), [("track_01_0m00s.mp4".to_owned(), 20)]);
}

#[test]
fn zoom_all_writes_beside_each_clip_a_zoom_of_the_same_frames() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let output = run(
        &[&videos, Path::new("-o"), &out, Path::new("--zoom"), Path::new("all")],
        "fr_FR.UTF-8",
    );
    assert!(output.status.success(), "{}", text(&output.stderr));
    let lengths = clips::lengths(&out.join("flight").join("extraits"));
    assert_eq!(
        lengths,
        [
            ("piste_01_0m00s.mp4".to_owned(), 20),
            ("piste_01_0m00s_zoom.mp4".to_owned(), 20)
        ]
    );
    let params: Value =
        serde_json::from_str(&fs::read_to_string(out.join("flight").join("params.json")).unwrap()).unwrap();
    assert_eq!(params["RenderConfig"]["zoom"], "all");
}

#[test]
fn a_track_that_is_neither_small_nor_faint_gets_no_zoom_by_default() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let names: Vec<String> = clips::lengths(&out.join("flight").join("clips"))
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(names, ["track_01_0m00s.mp4"]);
}

#[test]
fn annotated_writes_the_whole_video_beside_its_results() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let output = run(
        &[&videos, Path::new("-o"), &out, Path::new("--annotated")],
        "fr_FR.UTF-8",
    );
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        clips::frame_count(&out.join("flight").join("flight_annotee.mp4")),
        clips::frame_count(flight())
    );
    let params: Value =
        serde_json::from_str(&fs::read_to_string(out.join("flight").join("params.json")).unwrap()).unwrap();
    assert_eq!(params["RenderConfig"]["annotated"], true);
}

#[test]
fn an_annotated_video_of_an_earlier_run_is_removed_when_none_is_asked_for() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let folder = out.join("flight");
    fs::create_dir_all(&folder).unwrap();
    for name in ["flight_annotated.mp4", ".flight_annotated.mp4.partial"] {
        fs::write(folder.join(name), "").unwrap();
    }
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(!folder.join("flight_annotated.mp4").exists());
    assert!(!folder.join(".flight_annotated.mp4.partial").exists());
}
