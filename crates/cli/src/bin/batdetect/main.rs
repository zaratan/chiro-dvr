mod admission;
mod arguments;
mod board;
mod clock;
mod errors;
mod messages;
mod notify;
mod numbers;
mod parsing;
mod sheet;
mod status;
mod style;
mod texts;

use std::env;
use std::io::{self, IsTerminal, Write as _};
use std::path::{Path, PathBuf};
use std::process::{self, ExitCode};
use std::sync::Arc;

use batdetect::inventory::Stock;
use batdetect::jobs::Job;
use batdetect::language::system_language;
use batdetect::ledger::Ledger;
use batdetect::outcome::{self, Ran, run_all};
use batdetect::process::Wording as _;
use batdetect::process::{Versions, process};
use batdetect::tally::Tally;
use batdetect::written::{self, Wording};
use jiff::Timestamp;
use media::decode::Threads;

use crate::arguments::Request;
use crate::errors::report;
use crate::status::{Status, Video};
use crate::texts::Texts;

const INTERRUPTED: i32 = 130;

fn main() -> ExitCode {
    outcome::quiet_panics();
    let texts = Texts::new(system_language());
    if env::args_os().len() <= 1 {
        return if arguments::print_common_uses(texts).is_ok() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    let request = match arguments::parse(env::args_os(), texts) {
        Ok(request) => request,
        Err(error) => return report(texts, error),
    };
    let stock = match admission::admitted(&request, texts) {
        Ok(admitted) => admitted,
        Err(code) => return code,
    };
    let jobs = stock.jobs();
    let status =
        Arc::new(Status::new(texts, io::stderr().is_terminal(), jobs.len()).redoing_all(request.admission.force));
    let watched = Arc::clone(&status);
    let stopped = jobs.clone();
    let watching = ctrlc::set_handler(move || {
        settle(&stopped, watched.texts());
        let mut err = io::stderr().lock();
        match watched.interrupted(&mut err) {
            Ok(()) | Err(_) => process::exit(INTERRUPTED),
        }
    });
    if watching.is_err() {
        return ExitCode::FAILURE;
    }
    if writeln!(io::stderr(), "{}", texts.say(&messages::announcement(&stock))).is_err() {
        return ExitCode::FAILURE;
    }
    run(stock, &jobs, &status, &request)
}

fn run(stock: Stock, jobs: &[Job], status: &Status, request: &Request) -> ExitCode {
    let texts = status.texts();
    let options = &request.options;
    let ffmpeg = media::library::version();
    let versions = Versions {
        program: messages::version(),
        ffmpeg: &ffmpeg,
    };
    let wording = Wording {
        labels: &texts,
        headings: &texts,
        language: texts.language(),
    };
    let mut unwritten: Vec<files::error::Error> = written::roots(jobs)
        .iter()
        .filter_map(|root| written::write_page(root, jobs, 0, wording).err())
        .collect();
    let mut ledger = Ledger::default();
    let rows = run_all(
        jobs,
        |job, rank| {
            status.enter(rank);
            if let Some(earlier) = stock.earlier(job) {
                return Ok(Ran::Skipped(earlier));
            }
            let video = Video { status, rank, job };
            process(job, &texts, &video, Threads::Automatic, versions, options).map(Ran::Processed)
        },
        |job, rank, result| {
            unwritten.extend(ledger.note(job, result.as_ref().err(), options, Timestamp::now()));
            unwritten.extend(written::write_page(&written::root_of(job), jobs, rank.position, wording).err());
            let video = Video { status, rank, job };
            video.tell(&mut io::stderr().lock(), result, options)
        },
    );
    let Ok(rows) = rows else {
        settle(jobs, texts);
        return ExitCode::FAILURE;
    };
    let tally = Tally {
        rows,
        refused: stock.refused,
        unwritten,
    };
    let written = board::write(&mut io::stdout().lock(), &tally, options, texts);
    if cfg!(target_os = "macos")
        && !request.no_notify
        && io::stdout().is_terminal()
        && let Some(path) = env::var_os("PATH")
    {
        let pages: Vec<PathBuf> = tally
            .roots()
            .iter()
            .filter_map(|root| written::page_to_open(root, texts.language()))
            .collect();
        let pages: Vec<&Path> = pages.iter().map(PathBuf::as_path).collect();
        let calls = notify::calls(&board::title(&tally, texts), &pages);
        notify::run(&calls, &path);
    }
    match written {
        Ok(()) if tally.all_done() => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

fn settle(jobs: &[Job], texts: Texts) {
    let wording = Wording {
        labels: &texts,
        headings: &texts,
        language: texts.language(),
    };
    for root in written::roots(jobs) {
        match written::write_page(&root, jobs, jobs.len(), wording) {
            Ok(()) | Err(_) => {}
        }
    }
}
