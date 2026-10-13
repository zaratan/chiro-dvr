use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const VARIABLE: &str = "BATDETECT_VERSION";

fn main() -> io::Result<()> {
    let mut out = io::stdout().lock();
    writeln!(out, "cargo::rerun-if-env-changed={VARIABLE}")?;
    let given = env::var(VARIABLE).ok().filter(|version| !version.is_empty());
    let version = if let Some(version) = given {
        version
    } else if let Some(version) = described() {
        for watched in watched_paths() {
            writeln!(out, "cargo::rerun-if-changed={}", watched.display())?;
        }
        version
    } else {
        writeln!(out, "cargo::rerun-if-changed=build.rs")?;
        env!("CARGO_PKG_VERSION").to_owned()
    };
    writeln!(out, "cargo::rustc-env={VARIABLE}={version}")
}

fn git(arguments: &[&str]) -> Option<String> {
    let output = Command::new("git").args(arguments).output().ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim();
    (output.status.success() && !text.is_empty()).then(|| text.to_owned())
}

fn described() -> Option<String> {
    let described = git(&["describe", "--tags", "--match", "v[0-9]*", "--dirty", "--always"])?;
    Some(described.strip_prefix('v').unwrap_or(&described).to_owned())
}

fn watched_paths() -> Vec<PathBuf> {
    let own = git(&["rev-parse", "--absolute-git-dir"]).map(PathBuf::from);
    let common = git(&["rev-parse", "--path-format=absolute", "--git-common-dir"]).map(PathBuf::from);
    let branch = git(&["symbolic-ref", "-q", "HEAD"]);
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
        paths.extend(["crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"].map(|name| root.join(name)));
    }
    paths.into_iter().filter(|path| path.exists()).collect()
}
