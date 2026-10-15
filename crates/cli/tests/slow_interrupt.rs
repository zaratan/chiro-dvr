#[cfg(unix)]
#[test]
#[ignore = "slow"]
fn ctrl_c_during_the_detection_names_the_video_reached_and_exits_with_130_then_the_same_command_redoes_the_lot() {
    use std::io::{BufRead, BufReader, Read};
    use std::path::Path;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    use reference::fixture::fixture;

    let video = fixture("video_092_original_3m24-4m05.mp4").unwrap();
    let out = tempfile::tempdir().unwrap();
    let batdetect = |out: &Path| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_batdetect"));
        command
            .arg(&video)
            .arg("-o")
            .arg(out)
            .env_remove("LANGUAGE")
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .env("LANG", "fr_FR.UTF-8")
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        command
    };
    let mut child = batdetect(out.path()).spawn().unwrap();
    let mut errors = BufReader::new(child.stderr.take().unwrap());
    let mut seen = String::new();
    while !seen.ends_with("— détection\n") {
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
    assert_eq!(status.code(), Some(130), "{seen}");
    assert!(
        seen.ends_with("lot interrompu à la vidéo 1/1\u{a0}; relancer la même commande refait le lot\n"),
        "{seen}"
    );
    let again = batdetect(out.path()).output().unwrap();
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
