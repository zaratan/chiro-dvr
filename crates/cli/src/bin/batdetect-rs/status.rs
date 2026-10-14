use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};

use batdetect::jobs::Job;
use batdetect::outcome::{Failure, Rank};
use batdetect::process::{self, Processed};
use batdetect::progress::{Progress, Step};

use crate::messages;
use crate::texts::{Message, Texts};

#[cfg(test)]
mod tests;

const LINE_START: &str = "\r";

#[derive(Debug)]
pub(crate) struct Status {
    texts: Texts,
    terminal: bool,
    drawn: AtomicUsize,
}

impl Status {
    pub(crate) const fn new(texts: Texts, terminal: bool) -> Self {
        Self {
            texts,
            terminal,
            drawn: AtomicUsize::new(0),
        }
    }

    pub(crate) fn show(&self, out: &mut impl Write, line: &str) -> io::Result<()> {
        if !self.terminal {
            return writeln!(out, "{line}");
        }
        self.clear(out)?;
        write!(out, "{line}")?;
        self.drawn.store(line.chars().count(), Ordering::Relaxed);
        out.flush()
    }

    pub(crate) fn clear(&self, out: &mut impl Write) -> io::Result<()> {
        let drawn = self.drawn.swap(0, Ordering::Relaxed);
        if drawn == 0 {
            return Ok(());
        }
        write!(out, "{LINE_START}{:drawn$}{LINE_START}", "")
    }
}

#[derive(Debug)]
pub(crate) struct Video<'a> {
    pub(crate) status: &'a Status,
    pub(crate) rank: Rank,
    pub(crate) job: &'a Job,
}

impl Progress for Video<'_> {
    fn step(&self, step: Step) -> io::Result<()> {
        let message = messages::video_step(self.rank, self.job, step);
        self.status
            .show(&mut io::stderr().lock(), &self.status.texts.say(&message))
    }
}

impl Video<'_> {
    pub(crate) fn tell(
        &self,
        err: &mut impl Write,
        out: &mut impl Write,
        result: &Result<Processed, Failure>,
    ) -> io::Result<()> {
        let (job, rank, texts) = (self.job, self.rank, self.status.texts);
        self.status.clear(err)?;
        let mut say = |message: &Message| writeln!(err, "{}", texts.say(message));
        match result {
            Ok(processed) => {
                for kept in &processed.kept {
                    say(&messages::table_kept(job, kept))?;
                }
                for redone in &processed.rendered.redone {
                    say(&messages::redone(job, &redone.paths, &redone.cause))?;
                }
                say(&messages::video_done(rank, job, processed.tracks.len()))?;
                writeln!(out, "{}", texts.say(&messages::processed(job, processed.tracks.len())))
            }
            Err(failure) => {
                let cause = match failure {
                    Failure::Error(process::Error::TooManyTracks {
                        count,
                        limit,
                        annotated,
                    }) => messages::too_many_tracks(job, *count, *limit, *annotated),
                    Failure::Error(error) => messages::failed(job, error),
                    Failure::Panic(message) => messages::internal_error(job, message),
                };
                say(&cause)?;
                say(&messages::video_failed(rank, job))
            }
        }
    }
}
