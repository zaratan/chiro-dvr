#[path = "support/flight.rs"]
mod flight;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect");
const LINKS: [&str; 3] = [" src=\"", " href=\"", " data-src=\""];

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
fn evening(folder: &str, videos: &[&str]) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let night = root.path().join(folder);
    fs::create_dir(&night).unwrap();
    for video in videos {
        flight::write_flight(&night.join(video));
    }
    (root, night)
}

fn links(html: &str) -> Vec<String> {
    LINKS
        .iter()
        .flat_map(|attribute| html.split(attribute).skip(1))
        .filter_map(|rest| rest.split('"').next())
        .filter(|link| !link.starts_with('#'))
        .map(decoded)
        .collect()
}

#[cfg(test)]
fn decoded(link: &str) -> String {
    let mut bytes = link.bytes();
    let mut out = Vec::with_capacity(link.len());
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = char::from(bytes.next().unwrap()).to_digit(16).unwrap();
            let low = char::from(bytes.next().unwrap()).to_digit(16).unwrap();
            out.push(u8::try_from(high * 16 + low).unwrap());
        } else {
            out.push(byte);
        }
    }
    String::from_utf8(out).unwrap()
}

#[test]
fn the_page_beside_an_evening_with_an_awkward_name_links_only_files_that_exist() {
    let (_root, night) = evening("nuit 12 #1 été", &["vol été.mp4"]);
    fs::write(night.join("abimée.mp4"), "not a video").unwrap();
    let output = run(&[&night], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(1));
    let results = night.join("batdetect");
    let html = fs::read_to_string(results.join("resultats.html")).unwrap();
    assert!(html.contains("<title>Résultats de batdetect — nuit 12 #1 été</title>"));
    assert!(html.contains(r#"<section class="video failed""#));
    assert!(html.contains("abimée\u{a0}: ce fichier n&#39;est pas une vidéo lisible"));
    let linked = links(&html);
    assert!(linked.len() >= 3, "{linked:?}");
    for link in linked {
        assert!(results.join(&link).is_file(), "{link}");
    }
}

#[test]
fn the_page_follows_the_language_of_the_last_run_and_the_other_one_goes() {
    let (_root, night) = evening("nuit", &["vol.mp4"]);
    assert!(run(&[&night], "fr_FR.UTF-8").status.success());
    let results = night.join("batdetect");
    assert!(results.join("resultats.html").is_file());
    assert!(run(&[&night], "C").status.success());
    assert!(results.join("results.html").is_file());
    assert!(!results.join("resultats.html").exists());
}
