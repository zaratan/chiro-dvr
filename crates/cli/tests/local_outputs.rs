#[path = "support/tree.rs"]
mod tree;

use std::fs;
use std::path::Path;
use std::process::Command;

use reference::fixture::fixture;
use reference::location::recorded_outputs;

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect-rs");
const EXTRACT: &str = "video_092_original_3m24-4m05";

#[cfg(test)]
fn run(video: &Path, out: &Path, lang: &str) {
    let output = Command::new(BINARY)
        .arg(video)
        .arg("-o")
        .arg(out)
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", lang)
        .output()
        .unwrap();
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{errors}");
    assert_eq!(errors, "", "{lang}");
}

#[test]
#[ignore = "local"]
fn the_extract_in_french_then_in_english_in_the_same_folder_gives_the_files_of_the_reference_and_nothing_older() {
    let video = fixture(&format!("{EXTRACT}.mp4")).unwrap();
    let french = tree::tree(&recorded_outputs("fr", EXTRACT).unwrap());
    let english = tree::tree(&recorded_outputs("en", EXTRACT).unwrap());
    let out = tempfile::tempdir().unwrap();
    let folder = out.path().join(EXTRACT);
    run(&video, out.path(), "fr_FR.UTF-8");
    assert_eq!(tree::tree(&folder), french);
    fs::create_dir(folder.join("split")).unwrap();
    fs::write(folder.join("split").join("track_01_0m04s.mp4"), "").unwrap();
    fs::copy(
        folder.join(format!("{EXTRACT}_pistes.csv")),
        folder.join(format!("{EXTRACT}.tracks.csv")),
    )
    .unwrap();
    for earlier in [
        format!("{EXTRACT}.tracks.png"),
        format!("{EXTRACT}_resume_3_sur_8.png"),
        format!("{EXTRACT}_summary_2_of_2.png"),
        format!("{EXTRACT}_boxes.mp4"),
        format!(".{EXTRACT}_annotated.mp4.partial"),
    ] {
        fs::write(folder.join(earlier), "").unwrap();
    }
    run(&video, out.path(), "C");
    assert_eq!(tree::tree(&folder), english);
    run(&video, out.path(), "fr_FR.UTF-8");
    assert_eq!(tree::tree(&folder), french);
}
