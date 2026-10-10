use std::fs;
use std::io;
use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect-rs");

#[test]
fn version_names_the_lgpl_ffmpeg_it_links() {
    let versions = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../packaging/ffmpeg-versions")).unwrap();
    let pinned = versions
        .lines()
        .find_map(|line| line.strip_prefix("FFMPEG_VERSION="))
        .unwrap();
    let output = Command::new(BINARY).arg("--version").output().unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());
    assert!(text.starts_with("batdetect-rs 0.0.0\n"), "{text}");
    assert!(text.contains(&format!("FFmpeg {pinned}-")), "{text}");
    assert!(text.contains("— LGPL version 2.1 or later\n"), "{text}");
}

#[test]
fn without_arguments_the_help_goes_to_stderr_with_a_usage_code() {
    let output = Command::new(BINARY).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("Usage: batdetect-rs")
    );
}

#[test]
fn a_stdout_nobody_reads_gives_a_failure_code() {
    let (reader, writer) = io::pipe().unwrap();
    drop(reader);
    let status = Command::new(BINARY).arg("--version").stdout(writer).status().unwrap();
    assert_eq!(status.code(), Some(1));
}
