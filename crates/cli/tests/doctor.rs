use std::path::Path;
use std::process::{Command, Output};

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect");

#[cfg(test)]
fn doctor(folder: &Path, arguments: &[&str]) -> Output {
    Command::new(BINARY)
        .arg("doctor")
        .args(arguments)
        .current_dir(folder)
        .env("LANG", "fr_FR.UTF-8")
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .output()
        .unwrap()
}

#[test]
fn doctor_checks_the_current_folder_and_its_code_says_whether_something_is_to_fix() {
    let folder = tempfile::tempdir().unwrap();
    let output = doctor(folder.path(), &[]);
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(report.starts_with("batdetect "), "{report}");
    assert!(report.contains("encodeur\u{a0}: "), "{report}");
    assert!(report.contains("place libre\u{a0}: "), "{report}");
    assert!(report.contains("✓ écriture\u{a0}: permise dans "), "{report}");
    let problems = report.lines().any(|line| line.starts_with('✗'));
    assert_eq!(output.status.code(), Some(i32::from(problems)), "{report}");
}

#[cfg(unix)]
#[test]
fn doctor_in_a_folder_without_the_right_to_write_fails_with_what_to_do() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    let folder = tempfile::tempdir().unwrap();
    fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let output = doctor(folder.path(), &[]);
    fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let report = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(report.contains("✗ écriture\u{a0}: refusée dans "), "{report}");
    assert!(
        report.contains("à faire\u{a0}: choisir un dossier où l'écriture est permise, avec -o"),
        "{report}"
    );
}

#[test]
fn doctor_takes_no_argument() {
    let folder = tempfile::tempdir().unwrap();
    let output = doctor(folder.path(), &["video.mp4"]);
    assert_eq!(output.status.code(), Some(2));
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(errors.contains("doctor ne prend aucun argument"), "{errors}");
}
