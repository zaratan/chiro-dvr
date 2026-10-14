use std::any::Any;
use std::io;
use std::panic::{self, RefUnwindSafe};

use crate::jobs::Job;
use crate::process::{self, Processed};

#[cfg(test)]
mod tests;

const UNKNOWN_PANIC: &str = "unknown panic";

#[derive(Debug)]
pub enum Failure {
    Error(process::Error),
    Panic(String),
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
        let result = match panic::catch_unwind(|| work(job, rank)) {
            Ok(Ok(processed)) => Ok(processed),
            Ok(Err(error)) => Err(Failure::Error(error)),
            Err(payload) => Err(Failure::Panic(message(payload.as_ref()))),
        };
        report(job, rank, &result)?;
        outcomes.push(result.map(|processed| Done {
            tracks: processed.tracks.len(),
        }));
    }
    Ok(outcomes)
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
