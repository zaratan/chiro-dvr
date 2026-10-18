#![cfg(unix)]

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const SCRIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/reference-data.sh");
const README: &str = "README.md";
const BUCKET_README: &str = "# chiro-dvr-videos\n";
const DATA: &str = "data/video_001/tracks.json";
const DATA_CONTENT: &str = "{\"tracks\":[]}\n";
const DATA_SUM: &str = "6462753bc22c7cb172a97cd1b0b512e5d25fe89a0d4db28a3b7dae056144c83a";
const VIDEO: &str = "video_001.mp4";
const VIDEO_CONTENT: &str = "not a video\n";
const VIDEO_SUM: &str = "99b0882482e429d771a9ea6722240a1bc7a02af3590d836a0a3cf81f7ce66e40";
const FAKE_RCLONE: &str = r#"#!/bin/sh
set -eu
source=$2
dest=$3
shift 3
while [ $# -gt 0 ]; do
  case $1 in
    --files-from) list=$2; shift 2 ;;
    --log-file) log=$2; shift 2 ;;
    *) shift ;;
  esac
done
: > "$log"
while read -r name; do
  mkdir -p "$dest/$(dirname "$name")"
  cp "$source/$name" "$dest/$name"
  echo "INFO  : $name: Copied (new)" >> "$log"
done < "$list"
"#;

#[derive(Debug)]
struct Bucket {
    folder: TempDir,
}

#[cfg(test)]
impl Bucket {
    fn new() -> Self {
        let bucket = Self {
            folder: tempfile::tempdir().unwrap(),
        };
        for prefix in ["reference-v0.1.1", "reference-v0.2.0"] {
            bucket.write(&format!("bucket/{prefix}/{DATA}"), DATA_CONTENT);
            bucket.write(&format!("bucket/{prefix}/{README}"), BUCKET_README);
        }
        bucket.write(&format!("bucket/img_0000/{VIDEO}"), VIDEO_CONTENT);
        bucket.write(
            "sums/v0.1.1",
            &format!("{DATA_SUM}  reference-v0.1.1/{DATA}\n{VIDEO_SUM}  img_0000/{VIDEO}\n"),
        );
        bucket.write("sums/v0.2.0", &format!("{DATA_SUM}  reference-v0.2.0/{DATA}\n"));
        bucket.write("bin/rclone", FAKE_RCLONE);
        fs::set_permissions(bucket.path("bin/rclone"), fs::Permissions::from_mode(0o755)).unwrap();
        for target in ["reference", "videos", "v0.2.0"] {
            fs::create_dir(bucket.path(target)).unwrap();
        }
        bucket
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.folder.path().join(relative)
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn fetch(&self, with_v020: bool) -> Output {
        let mut path = vec![self.path("bin")];
        path.extend(env::split_paths(&env::var_os("PATH").unwrap()));
        let mut command = Command::new("bash");
        command
            .arg(SCRIPT)
            .env("PATH", env::join_paths(path).unwrap())
            .env("BATDETECT_DATA_REMOTE", self.path("bucket"))
            .env("BATDETECT_DATA_SUMS", self.path("sums/v0.1.1"))
            .env("BATDETECT_DATA_SUMS_V020", self.path("sums/v0.2.0"))
            .env("BATDETECT_REFERENCE_DIR", self.path("reference"))
            .env("BATDETECT_VIDEOS_DIR", self.path("videos"))
            .env_remove("BATDETECT_REFERENCE_V020_DIR");
        if with_v020 {
            command.env("BATDETECT_REFERENCE_V020_DIR", self.path("v0.2.0"));
        }
        command.output().unwrap()
    }
}

#[cfg(test)]
fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn a_folder_copied_whole_from_the_bucket_is_accepted_although_the_sums_leave_its_readme_out() {
    let bucket = Bucket::new();
    for target in ["reference", "videos", "v0.2.0"] {
        bucket.write(&format!("{target}/{README}"), BUCKET_README);
    }
    let output = bucket.fetch(true);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(read(&bucket.path(&format!("reference/{DATA}"))), DATA_CONTENT);
    assert_eq!(read(&bucket.path(&format!("v0.2.0/{DATA}"))), DATA_CONTENT);
    assert_eq!(read(&bucket.path(&format!("videos/{VIDEO}"))), VIDEO_CONTENT);
}

#[test]
fn a_readme_below_the_top_of_the_reference_folder_is_not_the_buckets_so_it_is_refused_before_any_download() {
    let bucket = Bucket::new();
    bucket.write(&format!("reference/data/{README}"), BUCKET_README);
    let output = bucket.fetch(false);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("holds files that are not in") && stderr.contains("data/README.md"),
        "{stderr}"
    );
    assert!(!bucket.path(&format!("reference/{DATA}")).exists());
    assert!(!bucket.path(&format!("videos/{VIDEO}")).exists());
}
