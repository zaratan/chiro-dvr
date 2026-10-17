use std::process::{Command, Output};

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect");

#[cfg(test)]
fn completion(arguments: &[&str]) -> Output {
    Command::new(BINARY)
        .arg("completion")
        .args(arguments)
        .env("LANG", "fr_FR.UTF-8")
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .output()
        .unwrap()
}

#[cfg(unix)]
#[test]
fn the_zsh_script_is_read_by_zsh_without_a_syntax_error() {
    use std::io::Write as _;
    use std::process::Stdio;

    let output = completion(&["zsh"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let mut zsh = Command::new("zsh")
        .arg("-n")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    zsh.stdin.take().unwrap().write_all(&output.stdout).unwrap();
    let checked = zsh.wait_with_output().unwrap();
    assert!(checked.status.success(), "{}", String::from_utf8_lossy(&checked.stderr));
}

#[test]
fn the_script_goes_alone_to_a_pipe_without_the_lines_for_the_profile() {
    let output = completion(&["zsh"]);
    assert!(output.stdout.starts_with(b"#compdef batdetect\n"));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn another_shell_is_refused_with_the_command_to_type() {
    let output = completion(&["bash"]);
    assert_eq!(output.status.code(), Some(2));
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(errors.contains("seul zsh est pris en charge"), "{errors}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
}
