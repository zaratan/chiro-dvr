#![cfg(target_os = "macos")]

#[path = "support/listing.rs"]
mod listing;

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use files::error::Error;
use files::names::Names;
use files::previous::remove_previous_outputs;
use files::table::{Column, Headings, write_tracks_csv};
use listing::left_in;
use tempfile::{TempDir, TempPath};
use types::frame::Pts;
use types::language::Language;
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;

const IMAGE: &str = "volume.dmg";
const MOUNT: &str = "mount";
const IMAGE_SIZE: &str = "64m";
const ANNOTATED: &str = "track,verdict\r\n1,yes\r\n";

#[derive(Debug)]
struct Debugged;

impl Headings for Debugged {
    fn heading(&self, column: Column, _language: Language) -> String {
        format!("{column:?}")
    }
}

#[derive(Debug)]
struct Attached {
    folder: TempDir,
}

impl Attached {
    fn mount(&self) -> PathBuf {
        self.folder.path().join(MOUNT)
    }
}

#[cfg(test)]
impl Attached {
    fn exfat() -> Self {
        let attached = Self {
            folder: tempfile::tempdir().unwrap(),
        };
        let image = attached.folder.path().join(IMAGE);
        hdiutil(&[
            "create".as_ref(),
            "-size".as_ref(),
            IMAGE_SIZE.as_ref(),
            "-fs".as_ref(),
            "ExFAT".as_ref(),
            image.as_os_str(),
        ]);
        fs::create_dir(attached.mount()).unwrap();
        hdiutil(&[
            "attach".as_ref(),
            "-nobrowse".as_ref(),
            "-mountpoint".as_ref(),
            attached.mount().as_os_str(),
            image.as_os_str(),
        ]);
        attached
    }
}

impl Drop for Attached {
    fn drop(&mut self) {
        let detached = Command::new("hdiutil")
            .arg("detach")
            .arg(self.mount())
            .arg("-force")
            .status();
        assert!(
            thread::panicking() || detached.is_ok_and(|status| status.success()),
            "the exFAT image is still attached at {}",
            self.mount().display()
        );
    }
}

#[cfg(test)]
fn hdiutil(arguments: &[&OsStr]) {
    let output = Command::new("hdiutil").args(arguments).output().unwrap();
    assert!(
        output.status.success(),
        "hdiutil {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(test)]
fn one_second() -> Timeline {
    let rate = FrameRate::new(Rational::new(30, 1).unwrap()).unwrap();
    let pts = (0..30).map(|frame| Some(Pts::new(frame))).collect();
    Timeline::new(pts, Rational::new(1, 30).unwrap(), rate).unwrap()
}

#[cfg(test)]
fn write_table(path: &Path) {
    write_tracks_csv(path, &[], &one_second(), Language::English, &Debugged).unwrap();
}

#[test]
#[ignore = "slow"]
fn macos_refuses_the_exclusive_rename_on_exfat_so_the_table_needs_another_way_not_to_overwrite() {
    let volume = Attached::exfat();
    let partial = volume.mount().join(".v_tracks.csv.partial");
    fs::write(&partial, "").unwrap();
    let failure = TempPath::try_from_path(&partial)
        .unwrap()
        .persist_noclobber(volume.mount().join("v_tracks.csv"))
        .unwrap_err();
    assert_ne!(failure.error.kind(), io::ErrorKind::AlreadyExists, "{}", failure.error);
}

#[test]
#[ignore = "slow"]
fn on_exfat_a_table_written_by_hand_is_removed_by_the_cleanup_and_written_again() {
    let volume = Attached::exfat();
    let dest = volume.mount().join("out");
    fs::create_dir(&dest).unwrap();
    let names = Names::new(Language::English);
    let table = names.tracks_csv(&dest, OsStr::new("v"));
    write_table(&table);
    fs::write(&table, ANNOTATED).unwrap();
    let kept = remove_previous_outputs(&dest, OsStr::new("v"), names).unwrap();
    assert_eq!(kept, []);
    assert_eq!(left_in(&dest), Vec::<String>::new());
    write_table(&table);
    assert_eq!(left_in(&dest), ["v_tracks.csv"]);
}

#[test]
#[ignore = "slow"]
fn on_exfat_a_table_is_never_written_over_an_annotated_one() {
    let volume = Attached::exfat();
    let dest = volume.mount().join("out");
    fs::create_dir(&dest).unwrap();
    let table = Names::new(Language::English).tracks_csv(&dest, OsStr::new("v"));
    fs::write(&table, ANNOTATED).unwrap();
    let error = write_tracks_csv(&table, &[], &one_second(), Language::English, &Debugged).unwrap_err();
    assert!(matches!(error, Error::Taken { .. }), "{error}");
    assert_eq!(fs::read_to_string(&table).unwrap(), ANNOTATED);
    assert_eq!(left_in(&dest), ["v_tracks.csv"]);
}
