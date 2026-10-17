use std::io::{self, Write};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use batdetect::eta::Evening;
use batdetect::jobs::Job;
use batdetect::options::Options;
use batdetect::outcome::{Failure, Ran, Rank};
use batdetect::progress::{Progress, Step};
use jiff::{SignedDuration, Zoned};

use crate::messages::{self, Homonyms};
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
    columns: fn() -> Option<usize>,
    pace: Mutex<Pace>,
    homonyms: Homonyms,
}

#[derive(Debug)]
struct Pace {
    evening: Evening,
    entered: Option<Instant>,
}

impl Pace {
    fn elapsed_s(&self) -> f64 {
        self.entered.map_or(0.0, |entered| entered.elapsed().as_secs_f64())
    }
}

const fn unmeasured() -> Option<usize> {
    None
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
            columns: unmeasured,
            pace: Mutex::new(Pace {
                evening: Evening::new(Vec::new()),
                entered: None,
            }),
            homonyms: Homonyms::none(),
        }
    }

    pub(crate) fn naming(self, jobs: &[Job]) -> Self {
        Self {
            homonyms: messages::homonyms(jobs),
            ..self
        }
    }

    pub(crate) fn fitted_to(self, columns: fn() -> Option<usize>) -> Self {
        Self { columns, ..self }
    }

    pub(crate) fn timed(self, evening: Evening) -> Self {
        Self {
            pace: Mutex::new(Pace { evening, entered: None }),
            ..self
        }
    }

    pub(crate) fn redoing_all(self, force: bool) -> Self {
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
        if let Ok(mut pace) = self.pace.lock() {
            pace.entered = Some(Instant::now());
        }
    }

    pub(crate) fn computed(&self, rank: Rank) {
        if let Ok(mut pace) = self.pace.lock() {
            let spent_s = pace.elapsed_s();
            pace.evening.computed(rank.position, spent_s);
        }
    }

    fn finish_during(&self, rank: Rank) -> Option<Zoned> {
        let left_s = {
            let pace = self.pace.lock().ok()?;
            pace.evening.left_during(rank.position, pace.elapsed_s())?
        };
        finish_in(left_s)
    }

    fn finish_after(&self, rank: Rank) -> Option<Zoned> {
        let left_s = self.pace.lock().ok()?.evening.left_after(rank.position)?;
        finish_in(left_s)
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
        let line = fitted(line, (self.columns)());
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

fn finish_in(left_s: f64) -> Option<Zoned> {
    Zoned::now()
        .checked_add(SignedDuration::try_from_secs_f64(left_s).ok()?)
        .ok()
}

fn fitted(line: &str, columns: Option<usize>) -> &str {
    let Some(width) = columns.map(|columns| columns.saturating_sub(1)) else {
        return line;
    };
    line.char_indices()
        .nth(width)
        .and_then(|(end, _)| line.get(..end))
        .unwrap_or(line)
}

#[derive(Debug)]
pub(crate) struct Video<'a> {
    pub(crate) status: &'a Status,
    pub(crate) rank: Rank,
    pub(crate) job: &'a Job,
}

impl Progress for Video<'_> {
    fn step(&self, step: Step) -> io::Result<()> {
        self.step_on(&mut io::stderr().lock(), step)
    }
}

impl Video<'_> {
    pub(crate) fn step_on(&self, out: &mut impl Write, step: Step) -> io::Result<()> {
        let (status, texts) = (self.status, self.status.texts);
        let line = texts.say(&messages::video_step(self.rank, self.name(), step));
        let line = match status.finish_during(self.rank) {
            Some(at) if status.terminal => texts.say(&messages::finish(line, &at)),
            _ => line,
        };
        status.show(out, &line)
    }

    fn name(&self) -> String {
        messages::shown_name(self.job, &self.status.homonyms)
    }

    fn last_line(&self, err: &mut impl Write, message: &Message) -> io::Result<()> {
        let (status, texts) = (self.status, self.status.texts);
        let line = texts.say(message);
        let line = match status.finish_after(self.rank) {
            Some(at) if !status.terminal => texts.say(&messages::finish(line, &at)),
            _ => line,
        };
        writeln!(err, "{line}")
    }

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
            Ok(Ran::Skipped(earlier)) => {
                self.last_line(err, &messages::video_skipped(rank, self.name(), earlier.tracks))
            }
            Ok(Ran::Processed(processed)) => {
                for redone in &processed.rendered.redone {
                    say(&messages::redone(job, &redone.paths, &redone.cause))?;
                }
                self.last_line(err, &messages::video_done(rank, self.name(), processed.tracks.len()))
            }
            Err(failure) => {
                for message in messages::failure_lines(job, failure, options) {
                    say(&message)?;
                }
                self.last_line(err, &messages::video_failed(rank, self.name(), failure))
            }
        }
    }
}
