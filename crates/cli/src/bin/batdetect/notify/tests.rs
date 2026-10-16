use std::ffi::OsString;
use std::path::Path;

use super::{Call, calls};

fn strings(arguments: &[&str]) -> Vec<OsString> {
    arguments.iter().map(OsString::from).collect()
}

const SCRIPT_ARGUMENTS: [&str; 6] = [
    "-e",
    "on run argv",
    "-e",
    "display notification (item 1 of argv) with title \"batdetect\"",
    "-e",
    "end run",
];

fn notification_of(title: &str) -> Call {
    let mut arguments = strings(&SCRIPT_ARGUMENTS);
    arguments.push(OsString::from(title));
    Call {
        program: "osascript",
        arguments,
    }
}

fn opening_of(page: &str) -> Call {
    Call {
        program: "open",
        arguments: strings(&[page]),
    }
}

#[test]
fn the_notification_carries_the_title_of_the_board_under_the_name_of_the_program() {
    assert_eq!(
        calls("Bilan : 1 vidéo, 1 réussie", &[]),
        [notification_of("Bilan : 1 vidéo, 1 réussie")]
    );
}

#[test]
fn a_title_with_quotes_and_script_words_stays_one_argument_outside_the_fixed_script() {
    let title = "Summary\" & (do shell script \"touch x\") & \"\\\nend run";
    assert_eq!(calls(title, &[]), [notification_of(title)]);
}

#[test]
fn the_results_page_of_each_folder_of_the_board_is_opened_once_after_the_notification_in_its_order() {
    let pages = [
        Path::new("/nuit-12-oct/batdetect/resultats.html"),
        Path::new("/resultats/resultats.html"),
    ];
    assert_eq!(
        calls("Summary: 3 videos", &pages),
        [
            notification_of("Summary: 3 videos"),
            opening_of("/nuit-12-oct/batdetect/resultats.html"),
            opening_of("/resultats/resultats.html"),
        ]
    );
}

#[test]
fn a_relative_page_whose_folder_starts_with_a_dash_is_not_read_as_an_option_of_open() {
    let pages = [Path::new("-R/results.html"), Path::new("out/results.html")];
    let opened: Vec<Call> = calls("Summary", &pages).into_iter().skip(1).collect();
    let expected = pages.map(|page| Call {
        program: "open",
        arguments: vec![Path::new(".").join(page).into_os_string()],
    });
    assert_eq!(opened, expected);
    assert!(
        opened
            .iter()
            .all(|call| !call.arguments[0].to_string_lossy().starts_with('-'))
    );
}

#[cfg(unix)]
static SPAWNING: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(unix)]
fn fake(folder: &Path, program: &str, log: &Path, status: u8) {
    use std::os::unix::fs::PermissionsExt;
    let script = folder.join(program);
    let body = format!(
        "#!/bin/sh\nprintf '%s\\n' {program} \"$@\" >> '{}'\nexit {status}\n",
        log.display()
    );
    std::fs::write(&script, body).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
fn logged(log: &Path) -> Vec<String> {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

#[cfg(unix)]
#[test]
fn the_tools_are_looked_up_on_the_given_path_and_receive_the_notification_then_each_page() {
    let _spawning = SPAWNING.lock().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let log = folder.path().join("calls.log");
    fake(folder.path(), "osascript", &log, 0);
    fake(folder.path(), "open", &log, 0);
    let pages = [
        Path::new("/nuit-12-oct/batdetect/resultats.html"),
        Path::new("/resultats/resultats.html"),
    ];
    let succeeded = super::run(&calls("Bilan : 2 vidéos", &pages), folder.path().as_os_str());
    assert_eq!(succeeded, [true, true, true]);
    let mut expected = vec!["osascript".to_owned()];
    expected.extend(SCRIPT_ARGUMENTS.map(str::to_owned));
    expected.extend(
        [
            "Bilan : 2 vidéos",
            "open",
            "/nuit-12-oct/batdetect/resultats.html",
            "open",
            "/resultats/resultats.html",
        ]
        .map(str::to_owned),
    );
    assert_eq!(logged(&log), expected);
}

#[cfg(unix)]
#[test]
fn a_failing_or_missing_tool_does_not_stop_the_pages_from_opening() {
    let _spawning = SPAWNING.lock().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let log = folder.path().join("calls.log");
    fake(folder.path(), "osascript", &log, 1);
    fake(folder.path(), "open", &log, 0);
    let pages = [Path::new("/resultats/results.html")];
    let failing = super::run(&calls("Summary", &pages), folder.path().as_os_str());
    assert_eq!(failing, [false, true]);
    let empty = tempfile::tempdir().unwrap();
    let missing = super::run(&calls("Summary", &pages), empty.path().as_os_str());
    assert_eq!(missing, [false, false]);
    let opened: Vec<String> = logged(&log).into_iter().skip_while(|line| line != "open").collect();
    assert_eq!(opened, ["open", "/resultats/results.html"]);
}
