#[cfg(test)]
#[cfg(unix)]
fn batdetect(out: &std::path::Path) -> std::process::Command {
    use std::process::{Command, Stdio};

    use reference::fixture::fixture;

    let video = fixture("video_092_original_3m24-4m05.mp4").unwrap();
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
}

#[cfg(unix)]
const A_MINUTE: std::time::Duration = std::time::Duration::from_secs(60);

#[cfg(unix)]
const AFTER_THE_END: std::time::Duration = std::time::Duration::from_secs(5);

#[cfg(unix)]
#[derive(Debug)]
struct Watched {
    child: std::process::Child,
    lines: std::sync::mpsc::Receiver<String>,
    seen: String,
}

#[cfg(test)]
#[cfg(unix)]
impl Watched {
    fn spawn(out: &std::path::Path) -> Self {
        use std::io::{BufRead, BufReader};
        use std::sync::mpsc;
        use std::thread;

        let mut child = batdetect(out).spawn().unwrap();
        let mut errors = BufReader::new(child.stderr.take().unwrap());
        let (sender, lines) = mpsc::channel();
        thread::spawn(move || {
            let mut bytes = Vec::new();
            while errors.read_until(b'\n', &mut bytes).unwrap_or(0) > 0 {
                if sender.send(String::from_utf8_lossy(&bytes).into_owned()).is_err() {
                    break;
                }
                bytes.clear();
            }
        });
        Self {
            child,
            lines,
            seen: String::new(),
        }
    }

    fn until_line(&mut self, ending: &str) {
        use std::sync::mpsc::RecvTimeoutError;
        use std::time::Instant;

        let deadline = Instant::now() + A_MINUTE;
        while !self.seen.ends_with(ending) {
            match self.next_line(deadline) {
                Ok(line) => self.seen.push_str(&line),
                Err(RecvTimeoutError::Timeout) => {
                    self.give_up(&format!("no line ending in {ending:?} within a minute"))
                }
                Err(RecvTimeoutError::Disconnected) => {
                    self.give_up(&format!("the error output closed before a line ending in {ending:?}"))
                }
            }
        }
    }

    fn finish(mut self) -> (std::process::ExitStatus, String) {
        use std::time::Instant;

        let Some(status) = ended_within(&mut self.child, Instant::now() + A_MINUTE) else {
            self.give_up("the command still runs after a minute");
        };
        if self.drained(Instant::now() + AFTER_THE_END) {
            return (status, self.seen);
        }
        panic!("the error output stays open after the command ended: {}", self.seen);
    }

    fn next_line(&self, deadline: std::time::Instant) -> Result<String, std::sync::mpsc::RecvTimeoutError> {
        self.lines
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
    }

    fn drained(&mut self, deadline: std::time::Instant) -> bool {
        use std::sync::mpsc::RecvTimeoutError;

        loop {
            match self.next_line(deadline) {
                Ok(line) => self.seen.push_str(&line),
                Err(RecvTimeoutError::Disconnected) => return true,
                Err(RecvTimeoutError::Timeout) => return false,
            }
        }
    }

    fn give_up(&mut self, why: &str) -> ! {
        use std::time::Instant;

        let stack = stack_of(self.child.id());
        match self.child.kill() {
            Ok(()) | Err(_) => {}
        }
        self.drained(Instant::now() + AFTER_THE_END);
        panic!("{why}: {}{stack}", self.seen);
    }
}

#[cfg(test)]
#[cfg(unix)]
fn ended_within(child: &mut std::process::Child, deadline: std::time::Instant) -> Option<std::process::ExitStatus> {
    use std::thread;
    use std::time::{Duration, Instant};

    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return Some(status);
        }
        if Instant::now() > deadline {
            return None;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
#[cfg(unix)]
fn stack_of(pid: u32) -> String {
    use std::process::{Command, Stdio};
    use std::time::Instant;

    let folder = tempfile::tempdir().unwrap();
    let report = folder.path().join("stack.txt");
    let Ok(mut sampling) = Command::new("sample")
        .args([&pid.to_string(), "3", "-mayDie", "-file"])
        .arg(&report)
        .stdout(Stdio::null())
        .spawn()
    else {
        return String::new();
    };
    if ended_within(&mut sampling, Instant::now() + A_MINUTE).is_none() {
        match sampling.kill() {
            Ok(()) | Err(_) => {}
        }
        return String::new();
    }
    std::fs::read_to_string(&report).map_or_else(|_| String::new(), |stack| format!("\n{stack}"))
}

#[cfg(test)]
#[cfg(unix)]
fn interrupted_at(step: &str, out: &std::path::Path) -> (std::process::ExitStatus, String) {
    use std::process::Command;

    let mut watched = Watched::spawn(out);
    watched.until_line(&format!("— {step}\n"));
    let signal = Command::new("kill")
        .args(["-INT", &watched.child.id().to_string()])
        .status()
        .unwrap();
    assert!(signal.success());
    watched.finish()
}

#[cfg(unix)]
#[test]
#[ignore = "slow"]
fn ctrl_c_during_the_detection_names_the_video_reached_and_exits_with_130_then_the_same_command_resumes_the_lot() {
    let out = tempfile::tempdir().unwrap();
    let (status, seen) = interrupted_at("détection", out.path());
    assert_eq!(status.code(), Some(130), "{seen}");
    assert!(
        seen.ends_with(
            "lot interrompu à la vidéo 1/1\u{a0}; relancer la même commande reprend là où le lot s'est arrêté\n"
        ),
        "{seen}"
    );
    let (again, said) = Watched::spawn(out.path()).finish();
    assert!(again.success(), "{said}");
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
    let (status, seen) = interrupted_at("extraits", out.path());
    assert_eq!(status.code(), Some(130), "{seen}");
    let folder = out.path().join("video_092_original_3m24-4m05");
    assert!(folder.join("video_092_original_3m24-4m05_pistes.csv").is_file());
    assert!(!folder.join("params.json").exists());
    let page = std::fs::read_to_string(out.path().join("resultats.html")).unwrap();
    assert!(page.contains("inachevée"), "{page}");
    assert!(!page.contains("pas encore traitée"), "{page}");
}
