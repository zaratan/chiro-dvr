use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use bench::noisy;
use bench::recorded::line;
use bench::synthetic::injection::Injected;
use reference::bench::{BenchReport, read_report, write_targets};
use tempfile::TempDir;

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect-bench");

#[cfg(test)]
fn run(arguments: &[&str]) -> Output {
    Command::new(BINARY).args(arguments).output().unwrap()
}

#[cfg(test)]
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[cfg(test)]
fn path(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[cfg(test)]
fn benched(root: &Path) -> (PathBuf, PathBuf) {
    let video = root.join("noisy.mp4");
    let targets = root.join("slow.jsonl.gz");
    noisy::write_video(&video).unwrap();
    let slow = Injected {
        target: noisy::slow_target(0, 10, 40).unwrap(),
        observations: Vec::new(),
    };
    write_targets(&targets, &[line(&slow)]).unwrap();
    let out = root.join("out");
    let output = run(&[path(&video), "--targets", path(&targets), "-o", path(&out)]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    (video, out.join("noisy"))
}

#[cfg(test)]
fn report(folder: &Path) -> BenchReport {
    read_report(&folder.join("bench.json")).unwrap()
}

#[test]
fn replayed_targets_are_benched_into_a_folder_named_after_the_video() {
    let root = TempDir::new().unwrap();

    let (_, folder) = benched(root.path());

    for file in [
        "bench.json",
        "run.json",
        "reference.jsonl.gz",
        "injected.jsonl.gz",
        "bats.jsonl.gz",
    ] {
        assert!(folder.join(file).is_file(), "{file}");
    }
    let written = report(&folder);
    assert!(written.bats[0].found);
    assert_eq!(written.sampling, serde_json::Value::Null);
    assert!(written.added["seconds"]["reference"].as_f64().unwrap() > 0.0);
}

#[test]
fn a_replay_evaluates_the_run_again_with_other_tracking_settings_and_leaves_it_untouched() {
    let root = TempDir::new().unwrap();
    let (_, folder) = benched(root.path());
    let before = fs::read(folder.join("bench.json")).unwrap();
    let replayed = root.path().join("replayed");

    let output = run(&["--replay", path(&folder), "-o", path(&replayed), "--min-hits", "50"]);

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!((report(&replayed).not_visible, report(&replayed).bats.len()), (1, 0));
    assert_eq!(report(&replayed).track["min_hits"], 50);
    assert_eq!(fs::read(folder.join("bench.json")).unwrap(), before);
    assert!(text(&output.stdout).starts_with("motion  amplitude"));
}

#[test]
fn a_bench_that_would_write_over_its_own_input_is_refused_before_any_detection() {
    let root = TempDir::new().unwrap();
    let (video, folder) = benched(root.path());
    let out = folder.parent().unwrap();
    let targets = folder.join("bats.jsonl.gz");
    let before = fs::read(&targets).unwrap();

    for arguments in [
        vec!["--replay", path(&folder), "-o", path(&folder)],
        vec![path(&video), "--targets", path(&targets), "-o", path(out)],
    ] {
        let output = run(&arguments);

        assert_eq!(output.status.code(), Some(1), "{arguments:?}");
        assert!(
            text(&output.stderr).contains("holds the input of the bench, which would be overwritten"),
            "{}",
            text(&output.stderr)
        );
    }
    let bare = Command::new(BINARY)
        .args([path(&video), "--targets", "bats.jsonl.gz", "-o", ".."])
        .current_dir(&folder)
        .output()
        .unwrap();
    assert_eq!(bare.status.code(), Some(1), "{}", text(&bare.stderr));
    assert_eq!(fs::read(&targets).unwrap(), before);
}

#[test]
fn a_bench_checked_against_its_own_recording_passes_its_guard() {
    let root = TempDir::new().unwrap();
    let (video, folder) = benched(root.path());
    let checked = root.path().join("checked");

    let output = run(&[path(&video), "--against", path(&folder), "-o", path(&checked)]);

    assert!(output.status.success(), "{}", text(&output.stderr));
    let guard = fs::read_to_string(checked.join("noisy_noisy").join("guard.txt")).unwrap();
    assert!(
        guard.starts_with(&format!("guard against {}: passed", folder.display())),
        "{guard}"
    );
    assert!(text(&output.stdout).contains(&guard), "{}", text(&output.stdout));
}

#[test]
fn drawn_targets_record_their_sampling() {
    let root = TempDir::new().unwrap();
    let video = root.path().join("noisy.mp4");
    noisy::write_video(&video).unwrap();
    let out = root.path().join("out");

    let output = run(&[
        path(&video),
        "-o",
        path(&out),
        "--per-class",
        "1",
        "--motions",
        "hunt",
        "--sigmas",
        "3",
        "--amplitudes",
        "-60",
    ]);

    assert!(output.status.success(), "{}", text(&output.stderr));
    let written = report(&out.join("noisy"));
    assert_eq!(written.sampling["per_class"], 1);
    assert_eq!(written.added["targets"]["drawn"]["seed"], 1);
}

#[test]
fn a_missing_video_fails_with_its_name_and_the_cause() {
    let root = TempDir::new().unwrap();
    let missing = root.path().join("absent.mp4");

    let output = run(&[path(&missing), "-o", path(&root.path().join("out"))]);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        text(&output.stderr)
            .lines()
            .any(|line| line.starts_with(&format!("{}: the reference detection failed", missing.display()))),
        "{}",
        text(&output.stderr)
    );
}

#[test]
fn the_help_is_shown_on_request_and_without_arguments() {
    let help = run(&["--help"]);
    let bare = run(&[]);

    assert!(help.status.success());
    assert!(text(&help.stdout).contains("--against <FOLDER>"));
    assert_eq!(bare.status.code(), Some(2));
}
