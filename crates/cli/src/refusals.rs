use std::path::PathBuf;

use files::error::Error;
use files::refusals::{RefusalRecord, RefusedInput, clear_refusals, record_refusals};
use jiff::Timestamp;

use crate::failure::REFERS_TO_DETAIL;
use crate::jobs::{Reason, Refused};

#[cfg(test)]
mod tests;

pub const VIDEOS_SEPARATOR: &str = ", ";

#[derive(Debug)]
pub struct Refusals {
    record: Option<RefusalRecord>,
    waiting: Vec<PathBuf>,
}

impl Refusals {
    pub fn new(roots: Vec<PathBuf>, refused: &[Refused], now: Timestamp) -> Self {
        Self {
            record: (!refused.is_empty()).then(|| RefusalRecord {
                date: now.to_string(),
                inputs: refused.iter().map(input).collect(),
            }),
            waiting: roots,
        }
    }

    pub fn note(&mut self) -> Vec<Error> {
        let (ready, waiting): (Vec<_>, Vec<_>) = self.waiting.drain(..).partition(|root| root.is_dir());
        self.waiting = waiting;
        ready
            .iter()
            .filter_map(|root| {
                match &self.record {
                    Some(record) => record_refusals(root, record),
                    None => clear_refusals(root),
                }
                .err()
            })
            .collect()
    }
}

pub fn input(refusal: &Refused) -> RefusedInput {
    let (cause, detail) = match &refusal.reason {
        Reason::Unopened { cause, detail } => (
            Some(cause.key().to_owned()),
            REFERS_TO_DETAIL.contains(&cause.key()).then(|| detail.clone()),
        ),
        _ => (None, None),
    };
    let videos = match &refusal.reason {
        Reason::SharedFolder { videos } => videos.iter().map(|video| video.display().to_string()).collect(),
        _ => Vec::new(),
    };
    RefusedInput {
        path: refusal.path.display().to_string(),
        reason: refusal.reason.key().to_owned(),
        cause,
        todo: refusal.reason.todo().to_owned(),
        videos,
        detail,
    }
}
