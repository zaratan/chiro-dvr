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

fn opening_of(folder: &str) -> Call {
    Call {
        program: "open",
        arguments: strings(&[folder]),
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
fn each_folder_of_the_board_is_opened_once_after_the_notification_in_its_order() {
    let roots = [Path::new("/nuit-12-oct/batdetect"), Path::new("/resultats")];
    assert_eq!(
        calls("Summary: 3 videos", &roots),
        [
            notification_of("Summary: 3 videos"),
            opening_of("/nuit-12-oct/batdetect"),
            opening_of("/resultats"),
        ]
    );
}

#[test]
fn a_relative_folder_whose_name_starts_with_a_dash_is_not_read_as_an_option_of_open() {
    let roots = [Path::new("-R"), Path::new("out")];
    let opened: Vec<Call> = calls("Summary", &roots).into_iter().skip(1).collect();
    assert_eq!(opened, [opening_of("./-R"), opening_of("./out")]);
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
fn the_tools_are_looked_up_on_the_given_path_and_receive_the_notification_then_each_folder() {
    let _spawning = SPAWNING.lock().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let log = folder.path().join("calls.log");
    fake(folder.path(), "osascript", &log, 0);
    fake(folder.path(), "open", &log, 0);
    let roots = [Path::new("/nuit-12-oct/batdetect"), Path::new("/resultats")];
    let succeeded = super::run(&calls("Bilan : 2 vidéos", &roots), folder.path().as_os_str());
    assert_eq!(succeeded, [true, true, true]);
    let mut expected = vec!["osascript".to_owned()];
    expected.extend(SCRIPT_ARGUMENTS.map(str::to_owned));
    expected.extend(
        [
            "Bilan : 2 vidéos",
            "open",
            "/nuit-12-oct/batdetect",
            "open",
            "/resultats",
        ]
        .map(str::to_owned),
    );
    assert_eq!(logged(&log), expected);
}

#[cfg(unix)]
#[test]
fn a_failing_or_missing_tool_does_not_stop_the_folders_from_opening() {
    let _spawning = SPAWNING.lock().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let log = folder.path().join("calls.log");
    fake(folder.path(), "osascript", &log, 1);
    fake(folder.path(), "open", &log, 0);
    let roots = [Path::new("/resultats")];
    let failing = super::run(&calls("Summary", &roots), folder.path().as_os_str());
    assert_eq!(failing, [false, true]);
    let empty = tempfile::tempdir().unwrap();
    let missing = super::run(&calls("Summary", &roots), empty.path().as_os_str());
    assert_eq!(missing, [false, false]);
    let opened: Vec<String> = logged(&log).into_iter().skip_while(|line| line != "open").collect();
    assert_eq!(opened, ["open", "/resultats"]);
}
