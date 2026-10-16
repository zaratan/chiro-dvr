use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};

use batdetect::jobs::Job;
use batdetect::options::Options;
use batdetect::outcome::{Failure, Ran, Rank};
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
    position: AtomicUsize,
    total: usize,
    resumes: bool,
}

impl Status {
    pub(crate) const fn new(texts: Texts, terminal: bool, total: usize) -> Self {
        Self {
            texts,
            terminal,
            drawn: AtomicUsize::new(0),
            position: AtomicUsize::new(1),
            total,
            resumes: true,
        }
    }

    pub(crate) const fn redoing_all(self, force: bool) -> Self {
        Self {
            resumes: !force,
            ..self
        }
    }

    pub(crate) const fn texts(&self) -> Texts {
        self.texts
    }

    pub(crate) fn enter(&self, rank: Rank) {
        self.position.store(rank.position, Ordering::Relaxed);
    }

    pub(crate) fn interrupted(&self, out: &mut impl Write) -> io::Result<()> {
        self.clear(out)?;
        let rank = Rank {
            position: self.position.load(Ordering::Relaxed),
            total: self.total,
        };
        let resumes = self.resumes;
        writeln!(out, "{}", self.texts.say(&Message::Interrupted { rank, resumes }))
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
        result: &Result<Ran, Failure>,
        options: &Options,
    ) -> io::Result<()> {
        let (job, rank, texts) = (self.job, self.rank, self.status.texts);
        self.status.clear(err)?;
        let mut say = |message: &Message| writeln!(err, "{}", texts.say(message));
        match result {
            Ok(Ran::Skipped(earlier)) => say(&messages::video_skipped(rank, job, earlier.tracks)),
            Ok(Ran::Processed(processed)) => {
                for redone in &processed.rendered.redone {
                    say(&messages::redone(job, &redone.paths, &redone.cause))?;
                }
                say(&messages::video_done(rank, job, processed.tracks.len()))
            }
            Err(failure) => {
                for message in messages::failure_lines(job, failure, options) {
                    say(&message)?;
                }
                say(&messages::video_failed(rank, job, failure))
            }
        }
    }
}
