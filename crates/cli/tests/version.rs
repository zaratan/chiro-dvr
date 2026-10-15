use std::fs;
use std::io;
use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect-rs");

fn english() -> Command {
    let mut command = Command::new(BINARY);
    command
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", "C");
    command
}

#[test]
fn version_names_the_lgpl_ffmpeg_it_links() {
    let versions = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../packaging/ffmpeg-versions")).unwrap();
    let pinned = versions
        .lines()
        .find_map(|line| line.strip_prefix("FFMPEG_VERSION="))
        .unwrap();
    let output = english().arg("--version").output().unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());
    assert!(
        text.starts_with(concat!("batdetect-rs ", env!("BATDETECT_VERSION"), "\n")),
        "{text}"
    );
    assert!(text.contains(&format!("FFmpeg {pinned}-")), "{text}");
    assert!(text.contains("— LGPL version 2.1 or later\n"), "{text}");
}

#[test]
fn without_arguments_the_common_uses_go_to_the_standard_output_with_a_success_code() {
    let output = english().output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let uses = String::from_utf8(output.stdout).unwrap();
    assert!(uses.starts_with("Detects and tracks bats"), "{uses}");
    assert!(uses.lines().any(|line| line == "Common uses:"), "{uses}");
    assert!(
        uses.lines().any(|line| line == "      batdetect-rs night-oct-12"),
        "{uses}"
    );
    assert!(
        uses.trim_end()
            .ends_with("All the options and the files produced: batdetect-rs --help"),
        "{uses}"
    );
    assert!(!uses.contains("Usage: "), "{uses}");
}

#[test]
fn an_output_folder_without_any_video_is_still_a_usage_error() {
    let output = english().args(["-o", "out"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_stdout_nobody_reads_gives_a_failure_code() {
    let (reader, writer) = io::pipe().unwrap();
    drop(reader);
    let status = english().arg("--version").stdout(writer).status().unwrap();
    assert_eq!(status.code(), Some(1));
}
