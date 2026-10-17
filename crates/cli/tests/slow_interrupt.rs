#[cfg(unix)]
const EXTRACT: &str = "video_092_original_3m24-4m05.mp4";

#[cfg(test)]
#[cfg(unix)]
fn batdetect(out: &std::path::Path, videos: &[&str]) -> std::process::Command {
    use std::process::{Command, Stdio};

    use reference::fixture::fixture;

    let mut command = Command::new(env!("CARGO_BIN_EXE_batdetect"));
    command
        .args(videos.iter().map(|video| fixture(video).unwrap()))
        .arg("-o")
        .arg(out)
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", "fr_FR.UTF-8")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    command
}

#[cfg(test)]
#[cfg(unix)]
fn interrupted_at(step: &str, out: &std::path::Path, videos: &[&str]) -> (std::process::ExitStatus, String) {
    use std::io::{BufRead, BufReader, Read};
    use std::process::Command;
    use std::thread;
    use std::time::{Duration, Instant};

    let mut child = batdetect(out, videos).spawn().unwrap();
    let mut errors = BufReader::new(child.stderr.take().unwrap());
    let mut seen = String::new();
    while !seen.ends_with(&format!("— {step}\n")) {
        assert!(errors.read_line(&mut seen).unwrap() > 0, "{seen}");
    }
    let signal = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(signal.success());
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            panic!("the command still runs a minute after Ctrl-C: {seen}");
        }
        thread::sleep(Duration::from_millis(50));
    };
    errors.read_to_string(&mut seen).unwrap();
    (status, seen)
}

#[cfg(unix)]
#[test]
#[ignore = "slow"]
fn ctrl_c_during_the_detection_names_the_video_reached_and_exits_with_130_then_the_same_command_resumes_the_lot() {
    let out = tempfile::tempdir().unwrap();
    let (status, seen) = interrupted_at("détection", out.path(), &[EXTRACT]);
    assert_eq!(status.code(), Some(130), "{seen}");
    assert!(
        seen.ends_with(
            "lot interrompu à la vidéo 1/1\u{a0}; relancer la même commande reprend là où le lot s'est arrêté\n"
        ),
        "{seen}"
    );
    let again = batdetect(out.path(), &[EXTRACT]).output().unwrap();
    assert!(again.status.success(), "{}", String::from_utf8_lossy(&again.stderr));
    let folder = out.path().join("video_092_original_3m24-4m05");
    let mut files = vec![folder.clone()];
    let mut partial = Vec::new();
    while let Some(path) = files.pop() {
        if path.is_dir() {
            files.extend(std::fs::read_dir(&path).unwrap().map(|entry| entry.unwrap().path()));
        } else if path.to_string_lossy().ends_with(".partial") {
            partial.push(path);
        }
    }
    assert_eq!(partial, Vec::<std::path::PathBuf>::new());
    assert!(folder.join("video_092_original_3m24-4m05_pistes.csv").is_file());
}

#[cfg(unix)]
#[test]
#[ignore = "slow"]
fn ctrl_c_during_the_clips_leaves_a_folder_without_params_json_that_reads_unfinished() {
    let out = tempfile::tempdir().unwrap();
    let (status, seen) = interrupted_at("extraits", out.path(), &[EXTRACT]);
    assert_eq!(status.code(), Some(130), "{seen}");
    let folder = out.path().join("video_092_original_3m24-4m05");
    assert!(folder.join("video_092_original_3m24-4m05_pistes.csv").is_file());
    assert!(!folder.join("params.json").exists());
    let page = std::fs::read_to_string(out.path().join("resultats.html")).unwrap();
    assert!(page.contains("inachevée"), "{page}");
    assert!(!page.contains("pas encore traitée"), "{page}");
}

#[cfg(unix)]
#[test]
#[ignore = "slow"]
fn ctrl_c_in_the_detection_of_the_first_video_of_two_leaves_a_page_where_it_is_unfinished_and_the_second_not_processed_yet()
 {
    let videos = tempfile::tempdir().unwrap();
    let out = videos.path().join("nuit");
    let (status, seen) = interrupted_at(
        "détection",
        &out,
        &[EXTRACT, "video_092_original_3m24-4m05_720x540.mp4"],
    );
    assert_eq!(status.code(), Some(130), "{seen}");
    let page = std::fs::read_to_string(out.join("resultats.html")).unwrap();
    let (_, cut) = page.split_once(">video_092_original_3m24-4m05</h2>").unwrap();
    let (cut, _) = cut.split_once("</section>").unwrap();
    assert!(
        cut.contains(">inachevée<") && !cut.contains("pas encore traitée"),
        "{page}"
    );
    let (_, later) = page.split_once(">video_092_original_3m24-4m05_720x540</h2>").unwrap();
    assert!(
        later
            .trim_start()
            .starts_with(r#"<span class="badge">pas encore traitée</span>"#),
        "{page}"
    );
    assert!(page.contains(r#"<span class="count">2 vidéos</span>"#), "{page}");
}
