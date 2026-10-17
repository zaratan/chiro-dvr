#[path = "support/clips.rs"]
mod clips;
#[path = "support/flight.rs"]
mod flight;
#[cfg(unix)]
#[path = "support/sparse.rs"]
mod sparse;
#[path = "support/tree.rs"]
mod tree;

use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Output, Stdio};
use std::sync::OnceLock;

use serde_json::Value;

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect");
const NOT_A_VIDEO_TODO: &str = "    à faire\u{a0}: s'il vient des jumelles, le recopier par le câble USB, sans passer par \
                                Stream Vision 2\u{a0}; sinon, le retirer du dossier";
const MISSING_TODO: &str =
    "    à faire\u{a0}: vérifier le chemin\u{a0}; le dossier ou la vidéo a peut-être été déplacé ou renommé";
const MISSING_TODO_EN: &str = "    what to do: check the path; the folder or the video may have been moved or renamed";
const NO_VIDEO_TODO: &str = "    à faire\u{a0}: vérifier que c'est le bon dossier\u{a0}: seules les vidéos posées directement \
                             dedans sont traitées, pas celles de ses sous-dossiers";
const NO_VIDEO_TODO_EN: &str = "    what to do: check it is the right folder: only the videos directly inside it are \
                                processed, not those of its subfolders";

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

#[cfg(test)]
fn occupied(folder: &Path) {
    fs::create_dir_all(folder.parent().unwrap()).unwrap();
    fs::write(folder, "").unwrap();
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
    assert!(table.starts_with("\u{feff}piste;début;fin;durée (s);verdict;points;corde (px);"));
    assert!(out.join("flight").join("params.json").is_file());
    assert!(
        text(&output.stdout).starts_with("Bilan\u{a0}: 1 vidéo, 1 réussie\n"),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn outside_a_french_locale_the_table_clips_and_summary_are_named_in_english_with_the_same_values_and_settings() {
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
    let read = |path: PathBuf| fs::read_to_string(path).unwrap();
    let (english_table, french_table) = (
        read(english.join("flight").join("flight_tracks.csv")),
        read(french.join("flight").join("flight_pistes.csv")),
    );
    assert!(
        english_table.starts_with("track,start,end,duration (s),verdict,points,"),
        "{english_table}"
    );
    let rows = |table: &str| table.lines().skip(1).map(str::to_owned).collect::<Vec<_>>();
    let french_rows: Vec<String> = rows(&french_table)
        .iter()
        .map(|row| row.replace(',', ".").replace(';', ","))
        .collect();
    assert_eq!(rows(&english_table), french_rows);
    assert_eq!(rows(&english_table).len(), 1);
    let settings = |folder: &Path| {
        let mut params: Value =
            serde_json::from_str(&fs::read_to_string(folder.join("flight").join("params.json")).unwrap()).unwrap();
        let object = params.as_object_mut().unwrap();
        assert!(object.remove("date").is_some_and(|date| date.is_string()));
        assert!(object.remove("elapsed_s").is_some_and(|elapsed| elapsed.is_f64()));
        let language = object.remove("language");
        let track = params["tracks"][0].as_object_mut().unwrap();
        let named = [language, track.remove("clip"), track.remove("summary")];
        (params, named)
    };
    let (english, french) = (settings(&english), settings(&french));
    assert_eq!(english.0, french.0);
    assert_eq!(
        [english.1, french.1],
        [
            ["en", "clips/track_01_0m00s.mp4", "flight_summary.png"].map(|name| Some(Value::from(name))),
            ["fr", "extraits/piste_01_0m00s.mp4", "flight_resume.png"].map(|name| Some(Value::from(name)))
        ]
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
        "state",
        "track_count",
        "elapsed_s",
        "tracks",
        "length_s",
        "not_analysed_s",
        "video_bytes",
        "language",
        "date",
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
fn a_file_named_like_a_video_that_cannot_be_opened_stops_the_others_before_any_work_with_what_to_do() {
    let (root, videos) = videos_with_a_flight(&["a.mp4"]);
    let fake = videos.join("zz_faux.mp4");
    fs::write(&fake, "not a video").unwrap();
    let out = root.path().join("out");
    for (lang, expected) in [
        (
            "fr_FR.UTF-8",
            format!(
                "erreur\u{a0}: aucune vidéo n'est traitée\u{a0}:\n  {}\u{a0}: ce fichier n'est pas une vidéo lisible, ou \
                 sa copie est incomplète\n{NOT_A_VIDEO_TODO}\n",
                fake.display()
            ),
        ),
        (
            "C",
            format!(
                "error: no video is processed:\n  {}: this file is not a readable video, or its copy is incomplete\n    \
                 what to do: if it comes from the binoculars, copy it again over the USB cable, not through Stream \
                 Vision 2; otherwise, move it out of the folder\n",
                fake.display()
            ),
        ),
    ] {
        let output = run(&[&videos, Path::new("-o"), &out], lang);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(text(&output.stderr), expected);
        assert!(!out.exists());
    }
}

#[test]
fn under_a_french_locale_a_failure_gives_its_cause_and_what_to_do_in_french_and_the_detail_in_english() {
    let (root, videos) = videos_with_a_flight(&["broken.mp4"]);
    let out = root.path().join("out");
    let dest = out.join("broken");
    occupied(&dest);
    let output = run(&[&videos.join("broken.mp4"), Path::new("-o"), &out], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(1));
    let errors = text(&output.stderr);
    let lines: Vec<&str> = errors.lines().collect();
    let cause = lines
        .iter()
        .position(|line| line.ends_with("broken.mp4\u{a0}: erreur inattendue"))
        .unwrap_or_else(|| panic!("{errors}"));
    assert_eq!(
        lines[cause + 1],
        "  à faire\u{a0}: le signaler avec le détail ci-dessous",
        "{errors}"
    );
    assert!(
        lines[cause + 2].starts_with(&format!("  détail\u{a0}: cannot create {}", dest.display())),
        "{errors}"
    );
}

#[test]
fn every_faulty_input_is_refused_at_once_before_any_work_one_line_each_with_what_to_do_without_the_usage_line() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let empty = videos.join("vide");
    fs::create_dir(&empty).unwrap();
    let readme = videos.join("README.md");
    fs::write(&readme, "").unwrap();
    let (missing, video, out) = (
        videos.join("absente.mp4"),
        videos.join("flight.mp4"),
        root.path().join("out"),
    );
    let (missing_name, readme_name, empty_name) = (missing.display(), readme.display(), empty.display());
    for (lang, expected) in [
        (
            "fr_FR.UTF-8",
            format!(
                "erreur\u{a0}: aucune vidéo n'est traitée\u{a0}:\n  {missing_name}\u{a0}: introuvable\n{MISSING_TODO}\n  \
                 {readme_name}\u{a0}: pas une vidéo (mp4, mov, avi, mkv ou m4v)\n    \
                 à faire\u{a0}: ne donner que des vidéos, ou des dossiers de vidéos\n  \
                 {empty_name}\u{a0}: aucune vidéo (mp4, mov, avi, mkv ou m4v) dans ce dossier\n{NO_VIDEO_TODO}\n"
            ),
        ),
        (
            "C",
            format!(
                "error: no video is processed:\n  {missing_name}: not found\n{MISSING_TODO_EN}\n  \
                 {readme_name}: not a video (mp4, mov, avi, mkv or m4v)\n    \
                 what to do: give only videos, or folders of videos\n  \
                 {empty_name}: no video (mp4, mov, avi, mkv or m4v) in this folder\n{NO_VIDEO_TODO_EN}\n"
            ),
        ),
    ] {
        let output = run(&[&missing, &readme, &empty, &video, Path::new("-o"), &out], lang);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(text(&output.stderr), expected);
        assert!(!out.exists());
    }
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
            "resultats.html",
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
fn a_video_that_does_not_exist_without_an_output_folder_is_refused_and_creates_no_folder() {
    let (root, videos) = videos_with_a_flight(&[]);
    let missing = root.path().join("faute").join("v.mp4");
    let output = run(&[&missing, &videos.join("absent.mp4")], "C");
    assert_eq!(output.status.code(), Some(2));
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
            "results.html",
        ]
    );
}

#[test]
fn a_folder_without_any_video_is_refused_by_name_with_the_extensions_looked_for_and_what_to_do() {
    let (root, videos) = videos_with_a_flight(&[]);
    let output = run(&[&videos, Path::new("-o"), &root.path().join("out")], "C");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        text(&output.stderr),
        format!(
            "error: no video (mp4, mov, avi, mkv or m4v) found in this folder:\n  {}\n{NO_VIDEO_TODO_EN}\n",
            videos.display()
        )
    );
}

#[test]
fn under_a_french_locale_the_help_and_the_refusals_are_in_french_with_their_usual_codes() {
    let (root, videos) = videos_with_a_flight(&[]);
    let help = run(&[Path::new("-h")], "fr_FR.UTF-8");
    assert_eq!(help.status.code(), Some(0));
    assert!(
        text(&help.stdout).contains("Utilisation\u{a0}: batdetect"),
        "{}",
        text(&help.stdout)
    );
    let refused = run(&[&videos, Path::new("-o"), &root.path().join("out")], "fr_FR.UTF-8");
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        text(&refused.stderr)
            .lines()
            .any(|line| line == "erreur\u{a0}: aucune vidéo (mp4, mov, avi, mkv ou m4v) trouvée dans ce dossier\u{a0}:"),
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
fn two_videos_that_would_share_an_output_folder_are_refused_by_that_folder_before_any_work_in_either_language() {
    let root = tempfile::tempdir().unwrap();
    let (first, second) = (root.path().join("a/night"), root.path().join("b/night"));
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    fs::copy(flight(), first.join("x.mp4")).unwrap();
    fs::copy(flight(), second.join("x.mp4")).unwrap();
    let out = root.path().join("out");
    let (first, second) = (first.join("x.mp4"), second.join("x.mp4"));
    let shared = out.join("night_x_mp4");
    for (lang, line) in [
        (
            "fr_FR.UTF-8",
            format!(
                "  {}\u{a0}: dossier de résultats commun à {}, {}",
                shared.display(),
                first.display(),
                second.display()
            ),
        ),
        (
            "C",
            format!(
                "  {}: results folder shared by {}, {}",
                shared.display(),
                first.display(),
                second.display()
            ),
        ),
    ] {
        let output = run(&[&first, &second, Path::new("-o"), &out], lang);
        assert_eq!(output.status.code(), Some(2));
        let errors = text(&output.stderr);
        assert_eq!(errors.lines().nth(1), Some(line.as_str()), "{errors}");
        assert!(!out.exists());
    }
}

#[cfg(unix)]
#[test]
fn a_folder_refused_to_read_is_refused_before_any_work_with_the_advice_given_for_a_video_refused_to_read() {
    use std::os::unix::fs::PermissionsExt;

    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    fs::set_permissions(&videos, fs::Permissions::from_mode(0o000)).unwrap();
    let french = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    let english = run(&[&videos, Path::new("-o"), &out], "C");
    fs::set_permissions(&videos, fs::Permissions::from_mode(0o755)).unwrap();
    for (output, refusal) in [
        (
            french,
            format!(
                "erreur\u{a0}: aucune vidéo n'est traitée\u{a0}:\n  {}\u{a0}: lecture refusée\n    à faire\u{a0}: \
                 vérifier ses droits\u{a0}; sur Mac, autoriser le Terminal dans Réglages Système, Confidentialité et \
                 sécurité, Fichiers et dossiers\n",
                videos.display()
            ),
        ),
        (
            english,
            format!(
                "error: no video is processed:\n  {}: reading refused\n    what to do: check its rights; on a Mac, \
                 allow the Terminal in System Settings, Privacy & Security, Files and Folders\n",
                videos.display()
            ),
        ),
    ] {
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(text(&output.stderr), refusal);
    }
    assert!(!out.exists());
}

#[test]
fn a_folder_with_a_video_and_the_hidden_copy_macos_drops_beside_it_on_exfat_processes_the_video_alone() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    fs::write(videos.join("._flight.mp4"), "AppleDouble").unwrap();
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let summary = text(&output.stdout);
    assert_eq!(summary.lines().next(), Some("Summary: 1 video, 1 done"), "{summary}");
    assert!(out.join("flight").join("flight_tracks.csv").is_file());
    assert!(!out.join(".batdetect").exists());
}

#[test]
fn a_folder_holding_only_hidden_copies_of_videos_is_refused_as_holding_no_video() {
    let (root, videos) = videos_with_a_flight(&[]);
    fs::write(videos.join("._flight.mp4"), "AppleDouble").unwrap();
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        text(&output.stderr),
        format!(
            "erreur\u{a0}: aucune vidéo (mp4, mov, avi, mkv ou m4v) trouvée dans ce dossier\u{a0}:\n  {}\n{NO_VIDEO_TODO}\n",
            videos.display()
        )
    );
    assert!(!out.exists());
}

#[test]
fn a_video_that_does_not_exist_stops_the_others_before_any_work() {
    let (root, videos) = videos_with_a_flight(&["ok.mp4"]);
    let out = root.path().join("out");
    let missing = videos.join("absent.mp4");
    let output = run(&[&missing, &videos.join("ok.mp4"), Path::new("-o"), &out], "C");
    assert_eq!(output.status.code(), Some(2));
    assert!(
        text(&output.stderr).contains(&format!("{}: not found", missing.display())),
        "{}",
        text(&output.stderr)
    );
    assert!(!out.exists());
}

#[test]
fn an_output_that_is_a_file_is_refused_by_name_before_any_work_with_another_folder_to_choose() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let file = root.path().join("out");
    fs::write(&file, "").unwrap();
    let output = run(&[&videos, Path::new("-o"), &file], "C");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        text(&output.stderr),
        format!(
            "error: no video is processed, the results cannot be written in this folder:\n  {}: not a folder\n    \
             what to do: choose another folder with -o\n",
            file.display()
        )
    );
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

#[cfg(target_os = "linux")]
#[test]
fn a_video_whose_name_is_not_utf8_is_processed_and_its_table_shown_on_the_page() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    fs::rename(
        videos.join("flight.mp4"),
        videos.join(OsStr::from_bytes(b"vol_\xe9.mp4")),
    )
    .unwrap();
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let dest = out.join(OsStr::from_bytes(b"vol_\xe9"));
    assert!(dest.join(OsStr::from_bytes(b"vol_\xe9_tracks.csv")).is_file());
    assert!(dest.join("params.json").is_file());
    assert!(dest.join("clips").join("track_01_0m00s.mp4").is_file());
    let page = fs::read_to_string(out.join("results.html")).unwrap();
    assert!(page.contains("vol_\u{fffd}_tracks.csv"), "{page}");
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

#[cfg(test)]
fn dated(folder: &Path, table: &str) -> Vec<String> {
    let prefix = format!("{table}_20");
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.starts_with(&prefix))
        .collect();
    names.sort();
    names
}

#[test]
fn a_table_annotated_by_hand_is_set_aside_and_named_in_the_summary_in_the_language_of_the_messages() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let folder = out.join("flight");
    fs::create_dir_all(&folder).unwrap();
    let annotated = "id;start;verdict\r\n1;0:00.00;oui\r\n";
    fs::write(folder.join("flight_pistes.csv"), annotated).unwrap();
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let [french] = dated(&folder, "flight_pistes").try_into().unwrap();
    let summary = text(&output.stdout);
    let lines: Vec<&str> = summary.lines().collect();
    let heading = lines
        .iter()
        .position(|line| *line == "Tableau mis de côté\u{a0}:")
        .unwrap_or_else(|| panic!("{summary}"));
    assert_eq!(
        lines[heading + 1],
        format!("  flight\u{a0}: flight_pistes.csv porte des verdicts, mis de côté sous {french}")
    );
    assert!(!summary.contains("laissé"), "{summary}");
    assert!(
        !text(&output.stderr).contains("flight_pistes"),
        "{}",
        text(&output.stderr)
    );
    assert_eq!(fs::read_to_string(folder.join(&french)).unwrap(), annotated);
    assert!(
        fs::read_to_string(folder.join("flight_pistes.csv"))
            .unwrap()
            .starts_with("\u{feff}piste;")
    );
    let unknown = "id,start,end,start_s,duration_s,hits,chord_px,path_px,speed_px_s,max_area_px,peak_amplitude,filled_frames,notes\r\n";
    fs::write(folder.join("flight_tracks.csv"), unknown).unwrap();
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let [english] = dated(&folder, "flight_tracks").try_into().unwrap();
    let summary = text(&output.stdout);
    assert!(
        summary.lines().any(|line| line
            == format!("  flight: flight_tracks.csv does not have the form of a tracks table, set aside as {english}")),
        "{summary}"
    );
    assert_eq!(fs::read_to_string(folder.join(&english)).unwrap(), unknown);
    assert_eq!(fs::read_to_string(folder.join(&french)).unwrap(), annotated);
    assert!(!folder.join("flight_pistes.csv").exists());
}

#[test]
fn a_verdict_written_in_the_table_of_an_earlier_run_is_set_aside_once_and_a_blank_table_written_beside_it() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    let folder = out.join("flight");
    let table = folder.join("flight_pistes.csv");
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let blank = fs::read(&table).unwrap();
    let annotated = String::from_utf8(blank.clone()).unwrap().replacen(";;", ";oui;", 1);
    fs::write(&table, &annotated).unwrap();
    let output = run(&[&videos, Path::new("-o"), &out, Path::new("--force")], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(
        text(&output.stdout).contains("Tableau mis de côté\u{a0}:"),
        "{}",
        text(&output.stdout)
    );
    let [aside] = dated(&folder, "flight_pistes").try_into().unwrap();
    assert_eq!(fs::read(folder.join(&aside)).unwrap(), annotated.as_bytes());
    assert_eq!(fs::read(&table).unwrap(), blank);
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(
        !text(&output.stdout).contains("Tableau mis de côté"),
        "{}",
        text(&output.stdout)
    );
    assert_eq!(dated(&folder, "flight_pistes"), [aside]);
    assert_eq!(fs::read(&table).unwrap(), blank);
}

#[cfg(unix)]
#[test]
fn a_folder_where_nothing_can_be_written_stops_every_video_before_any_work_and_says_what_to_do_in_either_language() {
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
                "erreur\u{a0}: aucune vidéo n'est traitée, les résultats ne peuvent pas s'écrire dans ce dossier\u{a0}:\n  \
                 {}\u{a0}: écriture refusée\n    à faire\u{a0}: vérifier les droits de ce dossier, ou en choisir un \
                 autre avec -o\n",
                refused.display()
            ),
        ),
        (
            english,
            format!(
                "error: no video is processed, the results cannot be written in this folder:\n  {}: writing refused\n    \
                 what to do: check the rights of this folder, or choose another one with -o\n",
                refused.display()
            ),
        ),
    ] {
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(text(&output.stderr), refusal);
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
    let (root, videos) = videos_with_a_flight(&["flight.mp4", "zz_broken.mp4"]);
    let out = root.path().join("out");
    occupied(&out.join("zz_broken"));
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
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
            "[2/2] zz_broken — suivi",
            "[2/2] zz_broken — échec",
        ]
    );
    let lines: Vec<&str> = errors.lines().collect();
    let cause = lines
        .iter()
        .position(|line| line.contains("zz_broken.mp4\u{a0}: erreur inattendue"))
        .unwrap();
    let failed = lines
        .iter()
        .position(|line| *line == "[2/2] zz_broken — échec")
        .unwrap();
    assert!(cause < failed, "{errors}");
}

#[test]
fn the_summary_alone_goes_to_the_standard_output_with_a_row_per_video_and_the_failures_below() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4", "zz_broken.mp4"]);
    let out = root.path().join("out");
    occupied(&out.join("zz_broken"));
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(1));
    let summary = text(&output.stdout);
    let lines: Vec<&str> = summary.lines().collect();
    assert_eq!(lines[0], "Bilan\u{a0}: 2 vidéos, 1 réussie, 1 en échec", "{summary}");
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
    assert!(lines.contains(&"1 vidéo sur 2 en échec\u{a0}:"), "{summary}");
    assert!(lines.contains(&"  zz_broken\u{a0}: erreur inattendue"), "{summary}");
    assert!(!summary.contains("détail\u{a0}:"), "{summary}");
    assert!(!summary.contains("[1/2]"), "{summary}");
    let errors = text(&output.stderr);
    assert!(errors.lines().any(|line| line == "[1/2] flight — 1 piste"), "{errors}");
    assert!(
        errors.lines().any(|line| line.starts_with("  détail\u{a0}: ")),
        "{errors}"
    );
}

#[test]
fn a_video_given_by_its_folder_and_again_under_another_writing_is_processed_once_under_one_title() {
    let (_root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let again = videos.join(".").join("flight.mp4");
    let output = run(&[&videos, &videos.join("flight.mp4"), &again], "fr_FR.UTF-8");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let summary = text(&output.stdout);
    let lines: Vec<&str> = summary.lines().collect();
    assert_eq!(lines[0], "Bilan\u{a0}: 1 vidéo, 1 réussie", "{summary}");
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

fn answerable(arguments: &[&Path]) -> io::Result<Child> {
    command(arguments, "fr_FR.UTF-8")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

fn write_answer(mut input: ChildStdin, answer: &str) -> io::Result<()> {
    match input.write_all(answer.as_bytes()) {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        written => written,
    }
}

fn run_answering(arguments: &[&Path], answer: &str) -> io::Result<Output> {
    let mut child = answerable(arguments)?;
    if let Some(input) = child.stdin.take() {
        write_answer(input, answer)?;
    }
    child.wait_with_output()
}

#[test]
fn an_answer_left_unread_by_a_command_that_already_stopped_is_not_a_write_error() {
    let root = tempfile::tempdir().unwrap();
    let mut child = answerable(&[&root.path().join("absente.mp4")]).unwrap();
    let input = child.stdin.take().unwrap();
    assert_eq!(child.wait_with_output().unwrap().status.code(), Some(2));
    write_answer(input, "o\n").unwrap();
}

#[test]
fn without_a_terminal_valid_videos_among_refused_inputs_are_not_processed_and_nothing_is_asked() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let (missing, out) = (videos.join("absente.mp4"), root.path().join("out"));
    let output = run_answering(&[&missing, &videos.join("flight.mp4"), Path::new("-o"), &out], "o\n").unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        text(&output.stderr),
        format!(
            "erreur\u{a0}: aucune vidéo n'est traitée\u{a0}:\n  {}\u{a0}: introuvable\n{MISSING_TODO}\n",
            missing.display()
        )
    );
    assert!(!out.exists());
}

#[test]
fn skip_refused_processes_the_valid_videos_without_asking_and_lists_the_refused_inputs_in_the_summary() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let (missing, out) = (videos.join("absente.mp4"), root.path().join("out"));
    let output = run_answering(
        &[
            &missing,
            &videos.join("flight.mp4"),
            Path::new("--skip-refused"),
            Path::new("-o"),
            &out,
        ],
        "n\n",
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let errors = text(&output.stderr);
    assert!(
        errors.starts_with(&format!(
            "erreur\u{a0}: une entrée est refusée\u{a0}:\n  {}\u{a0}: introuvable\n{MISSING_TODO}\n",
            missing.display()
        )),
        "{errors}"
    );
    assert!(!errors.contains("(O/n)"), "{errors}");
    assert!(out.join("flight").join("flight_pistes.csv").is_file());
    let summary = text(&output.stdout);
    let lines: Vec<&str> = summary.lines().collect();
    assert_eq!(
        lines[0], "Bilan\u{a0}: 1 vidéo, 1 réussie, 1 entrée refusée",
        "{summary}"
    );
    let refusal = format!("  {}\u{a0}: introuvable", missing.display());
    assert_eq!(
        lines[lines.len() - 3..],
        ["Entrée refusée, non traitée\u{a0}:", refusal.as_str(), MISSING_TODO],
        "{summary}"
    );
}

#[test]
fn skip_refused_does_not_turn_inputs_that_are_all_faulty_into_an_empty_run() {
    let (root, videos) = videos_with_a_flight(&[]);
    let (missing, out) = (videos.join("absente.mp4"), root.path().join("out"));
    let output = run(&[&missing, Path::new("--skip-refused"), Path::new("-o"), &out], "C");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        text(&output.stderr),
        format!(
            "error: no video is processed:\n  {}: not found\n{MISSING_TODO_EN}\n",
            missing.display()
        )
    );
    assert!(!out.exists());
}

#[test]
fn a_video_that_fails_at_its_turn_before_a_good_one_leaves_its_failure_beside_the_results() {
    let (_root, videos) = videos_with_a_flight(&["abimee.mp4", "flight.mp4"]);
    let dest = videos.join("batdetect").join("abimee");
    occupied(&dest);
    let output = run(&[&videos], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(1));
    let record = fs::read(videos.join("batdetect/.batdetect/echecs/abimee.json")).unwrap();
    let record: serde_json::Value = serde_json::from_slice(&record).unwrap();
    assert_eq!(record["video"], "abimee.mp4");
    assert_eq!(record["cause"], "unexpected");
}

#[test]
fn a_video_refused_at_the_start_in_a_folder_of_its_own_leaves_neither_a_folder_nor_a_title_there() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let traps = root.path().join("pieges");
    fs::create_dir(&traps).unwrap();
    fs::write(traps.join("abimee.mp4"), "not a video").unwrap();
    let output = run(&[&videos, &traps, Path::new("--skip-refused")], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(1));
    assert!(!traps.join("batdetect").exists());
    let summary = text(&output.stdout);
    let titles: Vec<&str> = summary.lines().filter(|line| line.ends_with("batdetect")).collect();
    assert_eq!(titles, [videos.join("batdetect").display().to_string()], "{summary}");
    assert!(
        summary.lines().any(|line| line == "Entrée refusée, non traitée\u{a0}:"),
        "{summary}"
    );
    let refusal = format!(
        "  {}\u{a0}: ce fichier n'est pas une vidéo lisible, ou sa copie est incomplète",
        traps.join("abimee.mp4").display()
    );
    assert!(summary.lines().any(|line| line == refusal), "{summary}");
}

#[test]
fn skip_refused_announces_and_processes_the_readable_video_and_counts_the_unreadable_one_among_the_refused() {
    let (root, videos) = videos_with_a_flight(&["a.mp4"]);
    fs::write(videos.join("zz_faux.mp4"), "not a video").unwrap();
    let out = root.path().join("out");
    let output = run(
        &[&videos, Path::new("--skip-refused"), Path::new("-o"), &out],
        "fr_FR.UTF-8",
    );
    assert_eq!(output.status.code(), Some(1));
    let errors = text(&output.stderr);
    let lines: Vec<&str> = errors.lines().collect();
    let announced = lines.iter().position(|line| *line == "1 vidéo, 2 s de vidéo");
    let first = lines.iter().position(|line| line.starts_with("[1/1] a — "));
    assert!(announced.is_some() && announced < first, "{errors}");
    assert!(out.join("a").join("a_pistes.csv").is_file());
    let summary = text(&output.stdout);
    assert_eq!(
        summary.lines().next(),
        Some("Bilan\u{a0}: 1 vidéo, 1 réussie, 1 entrée refusée"),
        "{summary}"
    );
}

#[test]
fn the_announcement_comes_once_before_the_first_video_and_counts_on_a_second_run_those_already_done() {
    let (root, videos) = videos_with_a_flight(&["a.mp4", "b.mp4", "c.mp4"]);
    let out = root.path().join("out");
    let arguments: [&Path; 3] = [&videos, Path::new("-o"), &out];
    let first = text(&run(&arguments, "fr_FR.UTF-8").stderr);
    let announcements: Vec<&str> = first.lines().filter(|line| line.contains(" de vidéo")).collect();
    assert_eq!(announcements, ["3 vidéos, 6 s de vidéo"], "{first}");
    assert!(first.starts_with("3 vidéos, 6 s de vidéo\n[1/3] "), "{first}");
    let again = text(&run(&arguments, "fr_FR.UTF-8").stderr);
    let announced = again.lines().next().unwrap_or_default();
    if reuses_earlier_results() {
        assert_eq!(announced, "3 vidéos, 6 s de vidéo, dont 3 déjà faites", "{again}");
    } else {
        assert_eq!(announced, "3 vidéos, 6 s de vidéo", "{again}");
    }
    let forced = text(&run(&[&videos, Path::new("-o"), &out, Path::new("--force")], "fr_FR.UTF-8").stderr);
    assert_eq!(forced.lines().next(), Some("3 vidéos, 6 s de vidéo"), "{forced}");
}

#[cfg(test)]
fn reuses_earlier_results() -> bool {
    !env!("BATDETECT_VERSION").ends_with("-dirty")
}

#[cfg(test)]
fn modified(folder: &Path) -> Vec<(String, std::time::SystemTime)> {
    tree::tree(folder)
        .into_iter()
        .filter(|name| !name.ends_with('/'))
        .map(|name| {
            let time = fs::metadata(folder.join(&name)).unwrap().modified().unwrap();
            (name, time)
        })
        .collect()
}

#[test]
fn the_same_command_again_skips_the_video_already_done_without_touching_its_folder_unless_forced_or_uncommitted() {
    let (root, videos) = videos_with_a_flight(&["flight.mp4"]);
    let out = root.path().join("out");
    assert!(run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8").status.success());
    let before = modified(&out.join("flight"));
    let again = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert!(again.status.success(), "{}", text(&again.stderr));
    let (title, line) = if reuses_earlier_results() {
        (
            "Bilan\u{a0}: 1 vidéo, 1 déjà faite\n",
            "[1/1] flight — déjà faite, 1 piste",
        )
    } else {
        ("Bilan\u{a0}: 1 vidéo, 1 réussie\n", "[1/1] flight — 1 piste")
    };
    assert!(text(&again.stdout).starts_with(title), "{}", text(&again.stdout));
    assert!(text(&again.stderr).contains(line), "{}", text(&again.stderr));
    assert_eq!(modified(&out.join("flight")) == before, reuses_earlier_results());
    let forced = run(&[&videos, Path::new("-o"), &out, Path::new("--force")], "fr_FR.UTF-8");
    assert!(
        text(&forced.stdout).starts_with("Bilan\u{a0}: 1 vidéo, 1 réussie\n"),
        "{}",
        text(&forced.stdout)
    );
    assert_ne!(modified(&out.join("flight")), before);
}

#[test]
fn a_lot_stopped_before_its_second_video_was_marked_finished_resumes_with_that_video_only() {
    let (root, videos) = videos_with_a_flight(&["a.mp4", "b.mp4"]);
    let out = root.path().join("out");
    assert!(run(&[&videos, Path::new("-o"), &out], "C").status.success());
    fs::remove_file(out.join("b").join("params.json")).unwrap();
    let first = modified(&out.join("a"));
    let again = run(&[&videos, Path::new("-o"), &out], "C");
    assert!(again.status.success(), "{}", text(&again.stderr));
    let title = if reuses_earlier_results() {
        "Summary: 2 videos, 1 done, 1 already done\n"
    } else {
        "Summary: 2 videos, 2 done\n"
    };
    assert!(text(&again.stdout).starts_with(title), "{}", text(&again.stdout));
    assert_eq!(modified(&out.join("a")) == first, reuses_earlier_results());
    assert!(out.join("b").join("params.json").is_file());
}

#[cfg(unix)]
#[test]
fn a_lot_too_big_for_the_disk_of_its_results_is_refused_before_any_work_naming_that_disk() {
    let (root, videos) = videos_with_a_flight(&["a.mp4"]);
    sparse::stretched_beyond_any_disk(&videos.join("a.mp4"));
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(2));
    let errors = text(&output.stderr);
    let lines: Vec<&str> = errors.lines().collect();
    assert_eq!(lines.len(), 3, "{errors}");
    assert_eq!(lines[0], "erreur\u{a0}: aucune vidéo n'est traitée\u{a0}:", "{errors}");
    assert!(
        lines[1].starts_with(&format!("  le disque qui contient {}\u{a0}: ", out.display()))
            && lines[1].ends_with(" Go libres"),
        "{errors}"
    );
    assert_eq!(
        lines[2],
        "    à faire\u{a0}: libérer de la place, ou écrire les résultats sur un autre disque avec -o\u{a0}; \
         --ignore-disk lance quand même le calcul",
        "{errors}"
    );
    assert!(!out.exists());
}

#[cfg(unix)]
#[test]
fn ignore_disk_starts_the_lot_and_a_video_whose_clips_cannot_fit_fails_as_a_full_disk_without_them() {
    let (root, videos) = videos_with_a_flight(&["a.mp4"]);
    sparse::stretched_beyond_any_disk(&videos.join("a.mp4"));
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out, Path::new("--ignore-disk")], "C");
    assert_eq!(output.status.code(), Some(1));
    let errors = text(&output.stderr);
    assert!(
        errors.lines().any(|line| line.ends_with("a.mp4: the disk is full")),
        "{errors}"
    );
    assert_eq!(
        text(&output.stdout).lines().next(),
        Some("Summary: 1 video, 1 failed"),
        "{}",
        text(&output.stdout)
    );
    assert!(!out.join("a").join("clips").exists());
    assert!(!out.join("a").join("params.json").exists());
}

#[cfg(unix)]
#[test]
fn skip_refused_starts_the_lot_despite_the_disk_and_does_not_count_the_disk_among_the_refused_inputs() {
    let (root, videos) = videos_with_a_flight(&["a.mp4"]);
    sparse::stretched_beyond_any_disk(&videos.join("a.mp4"));
    let out = root.path().join("out");
    let output = run(&[&videos, Path::new("-o"), &out, Path::new("--skip-refused")], "C");
    assert_eq!(output.status.code(), Some(1));
    let errors = text(&output.stderr);
    assert!(
        errors.lines().any(|line| line.starts_with("  the disk holding ")),
        "{errors}"
    );
    assert_eq!(
        text(&output.stdout).lines().next(),
        Some("Summary: 1 video, 1 failed"),
        "{}",
        text(&output.stdout)
    );
}

#[cfg(unix)]
#[test]
fn skip_refused_with_a_missing_input_and_a_disk_too_small_lists_both_but_records_only_the_input_as_refused() {
    let (root, videos) = videos_with_a_flight(&["a.mp4"]);
    sparse::stretched_beyond_any_disk(&videos.join("a.mp4"));
    let (missing, out) = (videos.join("absente.mp4"), root.path().join("out"));
    let output = run(
        &[
            &missing,
            &videos.join("a.mp4"),
            Path::new("-o"),
            &out,
            Path::new("--skip-refused"),
        ],
        "C",
    );
    assert_eq!(output.status.code(), Some(1));
    let errors = text(&output.stderr);
    assert!(errors.starts_with("error: one input is refused:\n"), "{errors}");
    assert!(
        errors
            .lines()
            .any(|line| line == "error: the disk of the results looks too small:"),
        "{errors}"
    );
    assert!(
        errors.lines().any(|line| line.ends_with("absente.mp4: not found")),
        "{errors}"
    );
    assert!(
        errors.lines().any(|line| line.starts_with("  the disk holding ")),
        "{errors}"
    );
    let summary = text(&output.stdout);
    assert_eq!(
        summary.lines().next(),
        Some("Summary: 1 video, 1 failed, 1 input refused"),
        "{summary}"
    );
    assert!(!summary.contains("the disk holding"), "{summary}");
    let recorded: Value =
        serde_json::from_str(&fs::read_to_string(files::refusals::refusals_path(&out)).unwrap()).unwrap();
    let paths: Vec<&Value> = recorded["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|input| &input["path"])
        .collect();
    assert_eq!(paths, [&Value::from(missing.display().to_string())], "{recorded}");
}

#[cfg(unix)]
#[test]
fn a_results_folder_where_nothing_can_be_written_is_refused_for_writing_only_even_on_a_disk_too_small() {
    use std::os::unix::fs::PermissionsExt;

    let (root, videos) = videos_with_a_flight(&["a.mp4"]);
    sparse::stretched_beyond_any_disk(&videos.join("a.mp4"));
    let locked = root.path().join("ro");
    fs::create_dir(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).unwrap();
    let out = locked.join("sortie");
    let output = run(&[&videos, Path::new("-o"), &out], "C");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(output.status.code(), Some(2));
    let errors = text(&output.stderr);
    assert!(
        errors.starts_with("error: no video is processed, the results cannot be written in this folder:\n"),
        "{errors}"
    );
    assert!(!errors.contains("the disk holding"), "{errors}");
}
