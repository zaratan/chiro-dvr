#[path = "support/clips.rs"]
mod clips;
#[path = "support/flight.rs"]
mod flight;
#[path = "support/tree.rs"]
mod tree;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::Value;

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect-rs");

#[cfg(test)]
fn command(arguments: &[&Path], lang: &str) -> Command {
    let mut command = Command::new(BINARY);
    command
        .args(arguments)
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("NO_COLOR")
        .env("LANG", lang);
    command
}

#[cfg(test)]
fn run(arguments: &[&Path], lang: &str) -> Output {
    command(arguments, lang).output().unwrap()
}

#[cfg(test)]
fn flight() -> &'static Path {
    static FLIGHT: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    let (_folder, path) = FLIGHT.get_or_init(|| {
        let folder = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
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

fn lines_naming<'a>(errors: &'a str, video: &str) -> Vec<&'a str> {
    errors.lines().filter(|line| line.contains(video)).collect()
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
    assert!(
        text(&output.stdout).starts_with("Bilan : 1 vidéo, 1 réussie\n"),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn outside_a_french_locale_the_table_is_named_in_english_and_holds_the_same_bytes() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let (english, french) = (root.path().join("en"), root.path().join("fr"));
    let output = run(&[&videos.join("flight.mp4"), Path::new("-o"), &english], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(
        text(&output.stdout).starts_with("Summary: 1 video, 1 done\n"),
        "{}",
        text(&output.stdout)
    );
    let output = run(&[&videos.join("flight.mp4"), Path::new("-o"), &french], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let read = |path: PathBuf| fs::read(path).unwrap();
    assert_eq!(
        read(english.join("flight").join("flight_tracks.csv")),
        read(french.join("flight").join("flight_pistes.csv"))
    );
    assert_eq!(
        read(english.join("flight").join("params.json")),
        read(french.join("flight").join("params.json"))
    );
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
    assert_eq!(params["version"], env!("BATDETECT_VERSION"));
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
    let failures: Vec<&str> = errors
        .lines()
        .filter(|line| !line.starts_with(' ') && line.contains(".mp4: "))
        .collect();
    assert_eq!(failures.len(), 1, "{errors}");
    assert!(
        failures[0].ends_with("broken.mp4: this file is not a readable video, or its copy is incomplete"),
        "{errors}"
    );
}

#[test]
fn under_a_french_locale_a_failure_gives_its_cause_and_what_to_do_in_french_and_the_detail_in_english() {
    let (root, videos) = videos_with_a_flight(&[]);
    fs::write(videos.join("broken.mp4"), "not a video").unwrap();
    let output = run(
        &[&videos.join("broken.mp4"), Path::new("-o"), &root.path().join("out")],
        "fr_FR.UTF-8",
    );
    assert_eq!(output.status.code(), Some(1));
    let errors = text(&output.stderr);
    let lines: Vec<&str> = errors.lines().collect();
    let cause = lines
        .iter()
        .position(|line| {
            line.ends_with("broken.mp4 : ce fichier n'est pas une vidéo lisible, ou sa copie est incomplète")
        })
        .unwrap_or_else(|| panic!("{errors}"));
    assert!(
        lines[cause + 1].starts_with("  à faire : s'il vient des jumelles, le recopier"),
        "{errors}"
    );
    assert!(
        lines[cause + 2].starts_with("  détail : the detection failed: cannot decode "),
        "{errors}"
    );
}

#[test]
fn a_video_that_does_not_exist_is_said_to_be_missing() {
    let (root, videos) = videos_with_a_flight(&[]);
    let missing = videos.join("absente.mp4");
    let output = run(&[&missing, Path::new("-o"), &root.path().join("out")], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(1));
    let errors = text(&output.stderr);
    assert!(
        errors
            .lines()
            .any(|line| line.ends_with("absente.mp4 : fichier introuvable")),
        "{errors}"
    );
}

#[test]
fn without_an_output_folder_the_results_go_to_a_batdetect_folder_beside_the_videos() {
    let (_root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let output = run(&[&videos], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        tree::tree(&videos.join("batdetect")),
        [
            "flight/",
            "flight/extraits/",
            "flight/extraits/piste_01_0m00s.mp4",
            "flight/flight_pistes.csv",
            "flight/flight_resume.png",
            "flight/params.json",
        ]
    );
}

#[test]
fn a_video_without_tracks_leaves_no_clips_folder_even_with_its_annotated_video() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let output = run(
        &[
            &videos,
            Path::new("-o"),
            &out,
            Path::new("--threshold"),
            Path::new("200"),
            Path::new("--annotated"),
        ],
        "fr_FR.UTF-8",
    );
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        tree::tree(&out.join("flight")),
        [
            "flight_annotee.mp4",
            "flight_pistes.csv",
            "flight_resume.png",
            "params.json"
        ]
    );
}

#[test]
fn a_video_that_does_not_exist_without_an_output_folder_creates_no_folder() {
    let (root, videos) = videos_with_a_flight(&[]);
    let missing = root.path().join("faute").join("v.mp4");
    let output = run(&[&missing, &videos.join("absent.mp4")], "C");
    assert_eq!(output.status.code(), Some(1));
    assert!(!root.path().join("faute").exists());
    assert!(!videos.join("batdetect").exists());
}

#[test]
fn without_an_output_folder_two_videos_of_the_same_name_in_two_folders_keep_their_name() {
    let root = tempfile::tempdir().unwrap();
    let nights = [root.path().join("nuit1"), root.path().join("nuit2")];
    for night in &nights {
        fs::create_dir(night).unwrap();
        fs::copy(flight(), night.join("v.mp4")).unwrap();
    }
    let output = run(&[&nights[0], &nights[1]], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    for night in &nights {
        assert!(night.join("batdetect").join("v").join("v_tracks.csv").is_file());
    }
}

#[test]
fn two_videos_whose_names_differ_only_by_case_keep_their_own_clips_in_one_output_folder() {
    let root = tempfile::tempdir().unwrap();
    let videos = [
        root.path().join("a").join("Vol.mp4"),
        root.path().join("b").join("vol.mp4"),
    ];
    for video in &videos {
        fs::create_dir_all(video.parent().unwrap()).unwrap();
        fs::copy(flight(), video).unwrap();
    }
    let out = root.path().join("out");
    let output = run(&[&videos[0], &videos[1], Path::new("-o"), &out], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        tree::tree(&out),
        [
            "a_Vol_mp4/",
            "a_Vol_mp4/Vol_summary.png",
            "a_Vol_mp4/Vol_tracks.csv",
            "a_Vol_mp4/clips/",
            "a_Vol_mp4/clips/track_01_0m00s.mp4",
            "a_Vol_mp4/params.json",
            "b_vol_mp4/",
            "b_vol_mp4/clips/",
            "b_vol_mp4/clips/track_01_0m00s.mp4",
            "b_vol_mp4/params.json",
            "b_vol_mp4/vol_summary.png",
            "b_vol_mp4/vol_tracks.csv",
        ]
    );
}

#[test]
fn a_folder_without_any_video_is_refused() {
    let (root, videos) = videos_with_a_flight(&[]);
    let output = run(&[&videos, Path::new("-o"), &root.path().join("out")], "C");
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("no video found"));
}

#[test]
fn under_a_french_locale_the_help_and_the_refusals_are_in_french_with_their_usual_codes() {
    let (root, videos) = videos_with_a_flight(&[]);
    let help = run(&[Path::new("-h")], "fr_FR.UTF-8");
    assert_eq!(help.status.code(), Some(0));
    assert!(
        text(&help.stdout).contains("Utilisation\u{a0}: batdetect-rs"),
        "{}",
        text(&help.stdout)
    );
    let refused = run(&[&videos, Path::new("-o"), &root.path().join("out")], "fr_FR.UTF-8");
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        text(&refused.stderr)
            .lines()
            .any(|line| line == "erreur\u{a0}: aucune vidéo trouvée"),
        "{}",
        text(&refused.stderr)
    );
}

#[test]
fn a_french_refusal_is_coloured_only_when_colours_are_forced() {
    let (root, videos) = videos_with_a_flight(&[]);
    let arguments: [&Path; 3] = [&videos, Path::new("-o"), &root.path().join("out")];
    let forced = command(&arguments, "fr_FR.UTF-8")
        .env("CLICOLOR_FORCE", "1")
        .output()
        .unwrap();
    assert!(
        text(&forced.stderr)
            .lines()
            .any(|line| line.starts_with("\u{1b}[1m\u{1b}[31merreur\u{a0}:\u{1b}[0m ")),
        "{}",
        text(&forced.stderr)
    );
    let plain = run(&arguments, "fr_FR.UTF-8");
    assert!(!text(&plain.stderr).contains('\u{1b}'));
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
fn an_output_that_is_a_file_is_refused_by_name_before_any_work() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let file = root.path().join("out");
    fs::write(&file, "").unwrap();
    let output = run(&[&videos, Path::new("-o"), &file], "C");
    assert_eq!(output.status.code(), Some(2));
    let errors = text(&output.stderr);
    let refusal = format!(
        "error: no video is processed, the results cannot be written in: {} (",
        file.display()
    );
    assert!(errors.lines().any(|line| line.starts_with(&refusal)), "{errors}");
    assert!(!errors.contains("flight.mp4"), "{errors}");
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

#[test]
fn the_settings_given_on_the_command_line_are_used_and_written_in_params_json() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let arguments: Vec<&Path> = [
        "--out-dir",
        "--threshold",
        "11",
        "--max-blobs",
        "0",
        "--clip-margin",
        "1",
        "--osd-region",
        "0,0,1,0.05",
    ]
    .iter()
    .map(Path::new)
    .collect();
    let output = run(
        &[
            videos.as_path(),
            arguments[0],
            &out,
            arguments[1],
            arguments[2],
            arguments[3],
            arguments[4],
            arguments[5],
            arguments[6],
            arguments[7],
            arguments[8],
        ],
        "C",
    );
    assert!(output.status.success(), "{}", text(&output.stderr));
    let params: Value =
        serde_json::from_str(&fs::read_to_string(out.join("flight").join("params.json")).unwrap()).unwrap();
    assert_eq!(params["DetectConfig"]["threshold"], 11.0);
    assert_eq!(params["DetectConfig"]["osd_regions"][0]["y1"], 0.05);
    assert_eq!(params["StabilityConfig"]["max_blobs"], 0);
    assert_eq!(params["RenderConfig"]["clip_margin_s"], 1.0);
}

#[test]
fn a_run_in_french_then_in_english_then_in_french_in_the_same_folder_leaves_only_the_last_one() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let french = [
        "extraits/",
        "extraits/piste_01_0m00s.mp4",
        "flight_pistes.csv",
        "flight_resume.png",
        "params.json",
    ];
    let english = [
        "clips/",
        "clips/track_01_0m00s.mp4",
        "flight_summary.png",
        "flight_tracks.csv",
        "params.json",
    ];
    for (lang, expected) in [("fr_FR.UTF-8", french), ("C", english), ("fr_FR.UTF-8", french)] {
        let output = run(&[&videos, Path::new("-o"), &out], lang);
        assert!(output.status.success(), "{}", text(&output.stderr));
        let errors = text(&output.stderr);
        assert_eq!(lines_naming(&errors, "flight.mp4"), Vec::<&str>::new(), "{lang}");
        assert_eq!(tree::tree(&out.join("flight")), expected, "{lang}");
    }
}

#[test]
fn a_table_annotated_by_hand_is_left_and_named_in_the_summary_in_the_language_of_the_messages() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let folder = out.join("flight");
    fs::create_dir_all(&folder).unwrap();
    let annotated = "id;start;verdict\r\n1;0:00.00;oui\r\n";
    fs::write(folder.join("flight_pistes.csv"), annotated).unwrap();
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let summary = text(&output.stdout);
    let lines: Vec<&str> = summary.lines().collect();
    let heading = lines
        .iter()
        .position(|line| *line == "Tableau laissé tel quel :")
        .unwrap_or_else(|| panic!("{summary}"));
    assert_eq!(
        lines[heading + 1],
        "  flight : flight_pistes.csv n'a pas la forme d'un tableau des pistes ; le nouveau tableau n'est pas écrit"
    );
    let errors = text(&output.stderr);
    assert!(!errors.contains("flight_pistes.csv"), "{errors}");
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let summary = text(&output.stdout);
    assert!(
        summary
            .lines()
            .any(|line| line == "  flight: flight_pistes.csv does not have the form of a tracks table"),
        "{summary}"
    );
    assert_eq!(fs::read_to_string(folder.join("flight_pistes.csv")).unwrap(), annotated);
    assert!(folder.join("flight_tracks.csv").is_file());
}

#[cfg(unix)]
#[test]
fn a_folder_where_nothing_can_be_written_stops_every_video_before_any_work_in_either_language() {
    use std::os::unix::fs::PermissionsExt;

    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let open = root.path().join("ouverte");
    fs::create_dir(&open).unwrap();
    fs::copy(flight(), open.join("autre.mp4")).unwrap();
    fs::set_permissions(&videos, fs::Permissions::from_mode(0o555)).unwrap();
    let french = run(&[&videos, &open], "fr_FR.UTF-8");
    let english = run(&[&videos, &open], "C");
    fs::set_permissions(&videos, fs::Permissions::from_mode(0o755)).unwrap();
    let refused = videos.join("batdetect");
    for (output, refusal) in [
        (
            french,
            format!(
                "erreur\u{a0}: aucune vidéo n'est traitée, les résultats ne peuvent pas s'écrire dans\u{a0}: {} (",
                refused.display()
            ),
        ),
        (
            english,
            format!(
                "error: no video is processed, the results cannot be written in: {} (",
                refused.display()
            ),
        ),
    ] {
        assert_eq!(output.status.code(), Some(2));
        let errors = text(&output.stderr);
        assert!(errors.lines().any(|line| line.starts_with(&refusal)), "{errors}");
        assert!(!errors.contains(&open.display().to_string()), "{errors}");
    }
    assert!(!open.join("batdetect").join("autre").exists());
}

fn video_lines(errors: &str, total: usize) -> Vec<&str> {
    let ranks: Vec<String> = (1..=total).map(|position| format!("[{position}/{total}] ")).collect();
    errors
        .lines()
        .filter(|line| ranks.iter().any(|rank| line.starts_with(rank.as_str())))
        .collect()
}

#[test]
fn redirected_each_video_shows_its_steps_in_order_then_its_track_count_or_its_failure() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    fs::write(videos.join("zz_broken.mp4"), "not a video").unwrap();
    let output = run(&[&videos, Path::new("-o"), &root.path().join("out")], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(1));
    let errors = text(&output.stderr);
    assert_eq!(
        video_lines(&errors, 2),
        [
            "[1/2] flight — détection",
            "[1/2] flight — suivi",
            "[1/2] flight — image résumé",
            "[1/2] flight — extraits",
            "[1/2] flight — 1 piste",
            "[2/2] zz_broken — détection",
            "[2/2] zz_broken — échec",
        ]
    );
    let lines: Vec<&str> = errors.lines().collect();
    let cause = lines
        .iter()
        .position(|line| line.contains("zz_broken.mp4 : ce fichier n'est pas une vidéo lisible"))
        .unwrap();
    let failed = lines
        .iter()
        .position(|line| *line == "[2/2] zz_broken — échec")
        .unwrap();
    assert!(cause < failed, "{errors}");
}

#[test]
fn the_summary_alone_goes_to_the_standard_output_with_a_row_per_video_and_the_failures_below() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    fs::write(videos.join("zz_broken.mp4"), "not a video").unwrap();
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(1));
    let summary = text(&output.stdout);
    let lines: Vec<&str> = summary.lines().collect();
    assert_eq!(lines[0], "Bilan : 2 vidéos, 1 réussie, 1 en échec", "{summary}");
    assert_eq!(lines[2], out.display().to_string(), "{summary}");
    assert!(lines[3].starts_with("  vidéo "), "{summary}");
    assert!(
        lines[4].starts_with("  flight ") && lines[4].ends_with(" s"),
        "{summary}"
    );
    assert!(
        lines[5].starts_with("  zz_broken ") && lines[5].contains(" échec "),
        "{summary}"
    );
    assert!(lines.contains(&"1 vidéo sur 2 en échec :"), "{summary}");
    assert!(
        lines.contains(&"  zz_broken : ce fichier n'est pas une vidéo lisible, ou sa copie est incomplète"),
        "{summary}"
    );
    assert!(!summary.contains("détail"), "{summary}");
    assert!(!summary.contains("[1/2]"), "{summary}");
    let errors = text(&output.stderr);
    assert!(errors.lines().any(|line| line == "[1/2] flight — 1 piste"), "{errors}");
    assert!(errors.lines().any(|line| line.starts_with("  détail : ")), "{errors}");
}

#[test]
fn a_video_given_by_its_folder_and_again_under_another_writing_is_processed_once_under_one_title() {
    let (_root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let again = videos.join(".").join("flight.mp4");
    let output = run(&[&videos, &videos.join("flight.mp4"), &again], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let summary = text(&output.stdout);
    let lines: Vec<&str> = summary.lines().collect();
    assert_eq!(lines[0], "Bilan : 1 vidéo, 1 réussie", "{summary}");
    let titles = lines.iter().filter(|line| line.ends_with("batdetect")).count();
    assert_eq!(titles, 1, "{summary}");
    assert!(
        videos
            .join("batdetect")
            .join("flight")
            .join("flight_pistes.csv")
            .is_file()
    );
}
