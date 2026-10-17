use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const CLEAN: &str = "clean";
const CHANGED: &str = "changed";
const WATCHED: [&str; 7] = [
    "crates",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    ".cargo",
    "packaging",
    "tests",
];

fn main() -> io::Result<()> {
    let mut out = io::stdout().lock();
    writeln!(out, "cargo::rerun-if-changed=build.rs")?;
    for watched in watched_paths() {
        writeln!(out, "cargo::rerun-if-changed={}", watched.display())?;
    }
    let described = output(
        "git",
        &["describe", "--tags", "--match", "v[0-9]*", "--dirty", "--always"],
    );
    let status = match status() {
        Some(listed) if listed.is_empty() => CLEAN,
        Some(_) => CHANGED,
        None => "",
    };
    let commit = output("git", &["rev-parse", "HEAD"]);
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let compiler = output(&rustc, &["--version"]);
    writeln!(
        out,
        "cargo::rustc-env=RECORD_DESCRIBED={}",
        described.unwrap_or_default()
    )?;
    writeln!(out, "cargo::rustc-env=RECORD_STATUS={status}")?;
    writeln!(out, "cargo::rustc-env=RECORD_COMMIT={}", commit.unwrap_or_default())?;
    writeln!(out, "cargo::rustc-env=RECORD_RUSTC={}", compiler.unwrap_or_default())
}

fn status() -> Option<String> {
    let rooted = WATCHED.map(|name| format!(":/{name}"));
    let mut arguments = vec!["status", "--porcelain", "--untracked-files=normal", "--"];
    arguments.extend(rooted.iter().map(String::as_str));
    run("git", &arguments)
}

fn output(program: &str, arguments: &[&str]) -> Option<String> {
    run(program, arguments).filter(|text| !text.is_empty())
}

fn run(program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).output().ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    output.status.success().then(|| text.trim().to_owned())
}

fn watched_paths() -> Vec<PathBuf> {
    let own = output("git", &["rev-parse", "--absolute-git-dir"]).map(PathBuf::from);
    let common = output("git", &["rev-parse", "--path-format=absolute", "--git-common-dir"]).map(PathBuf::from);
    let branch = output("git", &["symbolic-ref", "-q", "HEAD"]);
    let workspace = env::var("CARGO_MANIFEST_DIR").ok().and_then(|manifest| {
        Path::new(&manifest)
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
    });
    let mut paths: Vec<PathBuf> = own.iter().map(|dir| dir.join("HEAD")).collect();
    if let Some(dir) = &common {
        paths.extend([dir.join("refs").join("tags"), dir.join("packed-refs")]);
        paths.extend(branch.iter().map(|name| dir.join(name)));
    }
    if let Some(root) = &workspace {
        paths.extend(WATCHED.map(|name| root.join(name)));
    }
    paths.into_iter().filter(|path| path.exists()).collect()
}
