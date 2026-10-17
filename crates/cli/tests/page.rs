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
    let (_root, night) = evening("nuit 12 #1 été", &["vol été.mp4", "abimée.mp4"]);
    let results = night.join("batdetect");
    fs::create_dir_all(&results).unwrap();
    fs::write(results.join("abimée"), "").unwrap();
    let output = run(&[&night], "fr_FR.UTF-8");
    assert_eq!(output.status.code(), Some(1));
    let html = fs::read_to_string(results.join("resultats.html")).unwrap();
    assert!(html.contains("<title>Résultats de batdetect — nuit 12 #1 été</title>"));
    assert!(html.contains(r#"<section class="video failed""#));
    assert!(html.contains("abimée\u{a0}: erreur inattendue"));
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

#[test]
fn a_second_identical_run_keeps_the_page_complete_whether_it_skips_the_video_or_not() {
    let (_root, night) = evening("nuit", &["vol.mp4"]);
    assert!(run(&[&night], "fr_FR.UTF-8").status.success());
    let page = night.join("batdetect/resultats.html");
    let first = fs::read_to_string(&page).unwrap();
    assert!(run(&[&night], "fr_FR.UTF-8").status.success());
    let second = fs::read_to_string(&page).unwrap();
    assert!(second.contains(r#"<details class="video" id="v1" open>"#));
    assert_eq!(
        first.matches(r#"class="play""#).count(),
        second.matches(r#"class="play""#).count()
    );
}

#[test]
fn a_verdict_noted_in_the_table_shows_in_the_page_once_it_is_written_again() {
    let (_root, night) = evening("nuit", &["vol.mp4", "autre.mp4"]);
    assert!(run(&[&night.join("vol.mp4")], "fr_FR.UTF-8").status.success());
    let table = night.join("batdetect/vol/vol_pistes.csv");
    let written = fs::read_to_string(&table).unwrap();
    let mut lines: Vec<String> = written.split("\r\n").map(str::to_owned).collect();
    let verdict = lines[0].split(';').position(|cell| cell == "verdict").unwrap();
    let mut cells: Vec<&str> = lines[1].split(';').collect();
    cells[verdict] = "chauve-souris";
    lines[1] = cells.join(";");
    fs::write(&table, lines.join("\r\n")).unwrap();
    assert!(run(&[&night.join("autre.mp4")], "fr_FR.UTF-8").status.success());
    let page = fs::read_to_string(night.join("batdetect/resultats.html")).unwrap();
    assert!(page.contains(r#"<td class="verdict">chauve-souris</td>"#), "{page}");
}

#[test]
fn a_page_of_someone_else_in_the_way_of_two_videos_is_named_once_at_the_end() {
    let (_root, night) = evening("nuit", &["vol.mp4", "autre.mp4"]);
    let results = night.join("batdetect");
    fs::create_dir(&results).unwrap();
    let page = results.join("resultats.html");
    fs::write(&page, "<html>mes notes</html>").unwrap();
    let output = run(&[&night], "fr_FR.UTF-8");
    let summary = String::from_utf8(output.stdout).unwrap();
    assert!(
        summary.contains("\nFichier non écrit, la page de résultats peut être incomplète\u{a0}:\n"),
        "{summary}"
    );
    assert_eq!(summary.matches("resultats.html existe déjà").count(), 1, "{summary}");
    assert_eq!(fs::read_to_string(&page).unwrap(), "<html>mes notes</html>");
    assert!(results.join("vol").is_dir() && results.join("autre").is_dir());
}

#[test]
fn an_input_refused_at_the_start_shows_on_the_page_until_a_launch_without_refusal() {
    let (_root, night) = evening("nuit", &["vol.mp4"]);
    let video = night.join("vol.mp4");
    let missing = night.join("absente.mp4");
    let page = night.join("batdetect/resultats.html");
    run(&[&video, &missing, Path::new("--skip-refused")], "fr_FR.UTF-8");
    let html = fs::read_to_string(&page).unwrap();
    assert!(html.contains(r#"<section class="video failed refused" id="refused""#));
    assert!(html.contains("<p class=\"reason\">absente.mp4\u{a0}: introuvable</p>"));
    let shown = missing.display().to_string().replace('/', "/<wbr>");
    assert!(html.contains(&format!("à faire\u{a0}: vérifier le chemin {shown}\u{a0}; ")));
    assert!(html.contains(r##"<a class="count bad" href="#refused">1 entrée refusée</a>"##));
    assert!(run(&[&video], "fr_FR.UTF-8").status.success());
    let html = fs::read_to_string(&page).unwrap();
    assert!(!html.contains(r#"id="refused""#));
}
