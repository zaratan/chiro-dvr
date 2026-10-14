use std::any::Any;
use std::io;
use std::panic::{self, PanicHookInfo, RefUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use crate::jobs::Job;
use crate::process::{self, Processed};

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

#[derive(Debug)]
pub enum Failure {
    Error(process::Error),
    Panic(Panic),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Done {
    pub tracks: usize,
}

pub type Outcome = Result<Done, Failure>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rank {
    pub position: usize,
    pub total: usize,
}

pub fn run_all(
    jobs: &[Job],
    work: impl Fn(&Job, Rank) -> Result<Processed, process::Error> + RefUnwindSafe,
    mut report: impl FnMut(&Job, Rank, &Result<Processed, Failure>) -> io::Result<()>,
) -> io::Result<Vec<Outcome>> {
    let mut outcomes = Vec::with_capacity(jobs.len());
    for (job, position) in jobs.iter().zip(1..) {
        let rank = Rank {
            position,
            total: jobs.len(),
        };
        forget_panic();
        IN_VIDEO.store(true, Ordering::SeqCst);
        let caught = panic::catch_unwind(|| work(job, rank));
        IN_VIDEO.store(false, Ordering::SeqCst);
        let result = match caught {
            Ok(Ok(processed)) => Ok(processed),
            Ok(Err(error)) => Err(Failure::Error(error)),
            Err(payload) => Err(Failure::Panic(first_panic().unwrap_or_else(|| Panic {
                message: message(payload.as_ref()),
                location: None,
            }))),
        };
        report(job, rank, &result)?;
        outcomes.push(result.map(|processed| Done {
            tracks: processed.tracks.len(),
        }));
    }
    Ok(outcomes)
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

pub fn all_succeeded(outcomes: &[Outcome]) -> bool {
    outcomes.iter().all(Result::is_ok)
}

fn message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| UNKNOWN_PANIC.to_owned())
}
