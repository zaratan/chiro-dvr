use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(test)]
mod tests;

const NOTIFIER: &str = "osascript";
const OPENER: &str = "open";
const CURRENT_FOLDER: &str = ".";
const SCRIPT_FLAG: &str = "-e";
const SCRIPT: [&str; 3] = [
    "on run argv",
    concat!(
        "display notification (item 1 of argv) with title \"",
        env!("CARGO_BIN_NAME"),
        "\""
    ),
    "end run",
];

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Call {
    pub(crate) program: &'static str,
    pub(crate) arguments: Vec<OsString>,
}

pub(crate) fn calls(title: &str, pages: &[&Path]) -> Vec<Call> {
    std::iter::once(notification(title))
        .chain(pages.iter().map(|page| opening(page)))
        .collect()
}

fn notification(text: &str) -> Call {
    let script = SCRIPT.iter().flat_map(|line| [SCRIPT_FLAG, line]).map(OsString::from);
    Call {
        program: NOTIFIER,
        arguments: script.chain(std::iter::once(OsString::from(text))).collect(),
    }
}

fn opening(page: &Path) -> Call {
    Call {
        program: OPENER,
        arguments: vec![Path::new(CURRENT_FOLDER).join(page).into_os_string()],
    }
}

pub(crate) fn run(calls: &[Call], path: &OsStr) -> Vec<bool> {
    calls
        .iter()
        .map(|call| {
            Command::new(call.program)
                .args(&call.arguments)
                .env("PATH", path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        })
        .collect()
}
