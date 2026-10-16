#![cfg(windows)]

#[path = "support/flight.rs"]
mod flight;

use std::fs::OpenOptions;
use std::os::windows::fs::OpenOptionsExt;

use batdetect::failure::{Cause, cause};
use batdetect::jobs::Job;
use batdetect::options::Options;
use batdetect::process::{Versions, Wording, process};
use batdetect::progress::Silent;
use files::table::{Column, Headings};
use media::decode::Threads;
use summary::header::{Band, HeaderLine, Labels};
use types::language::Language;

const VERSIONS: Versions<'static> = Versions {
    program: "test",
    ffmpeg: "test",
};
const NO_SHARING: u32 = 0;

#[derive(Debug)]
struct Plain;

impl Labels for Plain {
    fn line(&self, line: &HeaderLine) -> String {
        format!("{line:?}")
    }

    fn band(&self, band: Band) -> String {
        format!("{band:?}")
    }
}

impl Headings for Plain {
    fn heading(&self, column: Column, _language: Language) -> String {
        format!("{column:?}")
    }
}

impl Wording for Plain {
    fn language(&self) -> Language {
        Language::English
    }
}

#[test]
#[ignore = "slow"]
fn a_summary_of_an_earlier_run_held_open_elsewhere_is_named_as_a_file_to_close() {
    let folder = tempfile::tempdir().unwrap();
    let job = Job {
        video: folder.path().join("flight.mp4"),
        dest: folder.path().join("out"),
    };
    flight::write_flight(&job.video);
    let options = Options::default();
    let summary = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &options)
        .unwrap()
        .summaries[0]
        .clone();
    let held = OpenOptions::new()
        .read(true)
        .share_mode(NO_SHARING)
        .open(&summary)
        .unwrap();
    let error = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &options).unwrap_err();
    drop(held);
    assert_eq!(cause(&error, &options), Cause::InUse { file: summary }, "{error}");
}
