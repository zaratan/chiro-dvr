use std::any::Any;
use std::io;
use std::panic::{self, PanicHookInfo, RefUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use files::previous::Kept;

use crate::jobs::Job;
use crate::process::{self, Extent, Processed};

#[cfg(test)]
mod tests;

const UNKNOWN_PANIC: &str = "unknown panic";

static IN_VIDEO: AtomicBool = AtomicBool::new(false);
static FIRST_PANIC: Mutex<Option<Panic>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panic {
    pub message: String,
    pub location: Option<String>,
}

impl Panic {
    pub fn detail(&self) -> String {
        match &self.location {
            Some(location) => format!("{} ({location})", self.message),
            None => self.message.clone(),
        }
    }
}

#[derive(Debug)]
pub enum Failure {
    Error(process::Error),
    Panic(Panic),
}

#[derive(Debug)]
pub enum State {
    Done {
        tracks: usize,
        extent: Extent,
        kept: Vec<Kept>,
    },
    TooManyTracks {
        count: usize,
        limit: usize,
        annotated: bool,
        extent: Extent,
        kept: Vec<Kept>,
    },
    Failed(Failure),
}

#[derive(Debug)]
pub struct Row {
    pub job: Job,
    pub elapsed: Duration,
    pub state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rank {
    pub position: usize,
    pub total: usize,
}

pub fn run_all(
    jobs: &[Job],
    work: impl Fn(&Job, Rank) -> Result<Processed, process::Error> + RefUnwindSafe,
    mut report: impl FnMut(&Job, Rank, &Result<Processed, Failure>) -> io::Result<()>,
) -> io::Result<Vec<Row>> {
    let mut rows = Vec::with_capacity(jobs.len());
    for (job, position) in jobs.iter().zip(1..) {
        let rank = Rank {
            position,
            total: jobs.len(),
        };
        forget_panic();
        let started = Instant::now();
        IN_VIDEO.store(true, Ordering::SeqCst);
        let caught = panic::catch_unwind(|| work(job, rank));
        IN_VIDEO.store(false, Ordering::SeqCst);
        let elapsed = started.elapsed();
        let result = match caught {
            Ok(Ok(processed)) => Ok(processed),
            Ok(Err(error)) => Err(Failure::Error(error)),
            Err(payload) => Err(Failure::Panic(first_panic().unwrap_or_else(|| Panic {
                message: message(payload.as_ref()),
                location: None,
            }))),
        };
        report(job, rank, &result)?;
        rows.push(Row {
            job: job.clone(),
            elapsed,
            state: state(result),
        });
    }
    Ok(rows)
}

fn state(result: Result<Processed, Failure>) -> State {
    match result {
        Ok(processed) => State::Done {
            tracks: processed.tracks.len(),
            extent: processed.extent,
            kept: processed.kept,
        },
        Err(Failure::Error(process::Error::TooManyTracks {
            count,
            limit,
            annotated,
            extent,
            kept,
        })) => State::TooManyTracks {
            count,
            limit,
            annotated,
            extent,
            kept,
        },
        Err(failure) => State::Failed(failure),
    }
}

pub fn quiet_panics() {
    let default = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if IN_VIDEO.load(Ordering::SeqCst) {
            keep(info);
        } else {
            default(info);
        }
    }));
}

fn keep(info: &PanicHookInfo<'_>) {
    let mut first = FIRST_PANIC.lock().unwrap_or_else(PoisonError::into_inner);
    if first.is_none() {
        *first = Some(Panic {
            message: info
                .payload_as_str()
                .map_or_else(|| UNKNOWN_PANIC.to_owned(), str::to_owned),
            location: info.location().map(ToString::to_string),
        });
    }
}

fn forget_panic() {
    *FIRST_PANIC.lock().unwrap_or_else(PoisonError::into_inner) = None;
}

fn first_panic() -> Option<Panic> {
    FIRST_PANIC.lock().unwrap_or_else(PoisonError::into_inner).take()
}

fn message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| UNKNOWN_PANIC.to_owned())
}
