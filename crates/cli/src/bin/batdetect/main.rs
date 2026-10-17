mod admission;
mod arguments;
mod board;
mod clock;
mod commands;
mod completion;
mod doctor;
mod errors;
mod messages;
mod notify;
mod numbers;
mod parsing;
mod sheet;
mod status;
mod style;
mod terminal;
mod texts;

use std::env;
use std::io::{self, IsTerminal, Write as _};
use std::path::{Path, PathBuf};
use std::process::{self, ExitCode};
use std::sync::{Arc, Mutex};

use batdetect::eta::Evening;
use batdetect::inventory::Stock;
use batdetect::jobs::{Job, Refused};
use batdetect::language::system_language;
use batdetect::ledger::Ledger;
use batdetect::outcome::{self, Ran, run_all};
use batdetect::process::Wording as _;
use batdetect::process::{Versions, process};
use batdetect::refusals::Refusals;
use batdetect::tally::Tally;
use batdetect::written::{self, Run, Wording};
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
    if let Some(code) = commands::other_command(texts) {
        return code;
    }
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
    let status = Arc::new(
        Status::new(texts, io::stderr().is_terminal(), jobs.len())
            .redoing_all(request.admission.force)
            .naming(&jobs)
            .fitted_to(terminal::stderr_columns)
            .timed(Evening::new(stock.frames_to_compute())),
    );
    let watched = Arc::clone(&status);
    let ledger = Arc::new(Mutex::new(Ledger::default()));
    let kept = Arc::clone(&ledger);
    let stopped = jobs.clone();
    let launched = Timestamp::now();
    let refused = stock.refused.clone();
    let watching = ctrlc::set_handler(move || {
        settle(&stopped, &refused, launched, watched.texts(), watched.position(), &kept);
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
    run(stock, &jobs, &status, &request, launched, &ledger)
}

fn run(
    stock: Stock,
    jobs: &[Job],
    status: &Status,
    request: &Request,
    launched: Timestamp,
    ledger: &Mutex<Ledger>,
) -> ExitCode {
    let texts = status.texts();
    let options = &request.options;
    let ffmpeg = media::library::version();
    let versions = Versions {
        program: messages::version(),
        ffmpeg: &ffmpeg,
    };
    let wording = Wording {
        labels: &texts,
        language: texts.language(),
    };
    let roots = written::roots(jobs);
    let mut refusals = Refusals::new(roots.clone(), &stock.refused, launched);
    let mut unwritten = refusals.note();
    unwritten.extend(
        roots
            .iter()
            .filter_map(|root| written::write_page(root, jobs, Run::Going { finished: 0 }, wording).err()),
    );
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
            if let Ok(mut ledger) = ledger.lock() {
                unwritten.extend(ledger.note(job, result.as_ref().err(), options, Timestamp::now()));
            }
            unwritten.extend(refusals.note());
            let run = Run::Going {
                finished: rank.position,
            };
            unwritten.extend(written::write_page(&written::root_of(job), jobs, run, wording).err());
            if matches!(result, Ok(Ran::Processed(_))) {
                status.computed(rank);
            }
            let video = Video { status, rank, job };
            video.tell(&mut io::stderr().lock(), result, options)
        },
    );
    let Ok(rows) = rows else {
        settle(jobs, &stock.refused, launched, texts, status.position(), ledger);
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

fn settle(
    jobs: &[Job],
    refused: &[Refused],
    launched: Timestamp,
    texts: Texts,
    reached: usize,
    ledger: &Mutex<Ledger>,
) {
    let wording = Wording {
        labels: &texts,
        language: texts.language(),
    };
    match written::create_reached_root(jobs, reached) {
        Ok(()) | Err(_) => {}
    }
    if let Ok(mut ledger) = ledger.lock() {
        ledger.flush();
    }
    let roots = written::roots(jobs);
    Refusals::new(roots.clone(), refused, launched).note();
    for root in roots {
        match written::write_page(&root, jobs, Run::Stopped { reached }, wording) {
            Ok(()) | Err(_) => {}
        }
    }
}
