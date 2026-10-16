use std::ffi::OsString;
use std::path::{Path, PathBuf};

use files::error::Error;
use files::failures::{FailureRecord, clear_failure, record_failure};
use jiff::Timestamp;

use crate::failure::{self, Cause, INTERNAL};
use crate::jobs::Job;
use crate::options::Options;
use crate::outcome::Failure;
use crate::process;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub struct Ledger {
    waiting: Vec<(PathBuf, OsString, FailureRecord)>,
}

impl Ledger {
    pub fn note(&mut self, job: &Job, failure: Option<&Failure>, options: &Options, now: Timestamp) -> Vec<Error> {
        let root = job.dest.parent().unwrap_or(&job.dest).to_path_buf();
        let folder = job.dest.file_name().unwrap_or_default().to_os_string();
        let mut errors = Vec::new();
        match failed(job, failure, options, now) {
            Some(record) => self.waiting.push((root, folder, record)),
            None => errors.extend(clear_failure(&root, &folder).err()),
        }
        let (ready, waiting): (Vec<_>, Vec<_>) = self.waiting.drain(..).partition(|(root, _, _)| root.is_dir());
        self.waiting = waiting;
        errors.extend(
            ready
                .iter()
                .filter_map(|(root, folder, record)| record_failure(root, folder, record).err()),
        );
        errors
    }
}

fn failed(job: &Job, failure: Option<&Failure>, options: &Options, now: Timestamp) -> Option<FailureRecord> {
    let (cause, place, detail) = match failure? {
        Failure::Error(process::Error::TooManyTracks { .. }) => return None,
        Failure::Error(error) => {
            let cause = failure::cause(error, options);
            let place = match &cause {
                Cause::InUse { file } => file.as_path(),
                _ => job.dest.as_path(),
            };
            (cause.key(), shown(place), failure::chain(error))
        }
        Failure::Panic(panic) => (INTERNAL, shown(&job.dest), panic.detail()),
    };
    Some(FailureRecord {
        video: shown(job.video.file_name().unwrap_or_default()),
        cause: cause.to_owned(),
        place,
        detail: Some(detail),
        date: now.to_string(),
    })
}

fn shown(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().into_owned()
}
