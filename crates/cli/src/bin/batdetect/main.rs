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

use batdetect::consent::{Asking, Choice};
use batdetect::inventory::{Stock, take_stock};
use batdetect::jobs::{self, Job, Refused, plan};
use batdetect::language::system_language;
use batdetect::ledger::Ledger;
use batdetect::outcome::{self, Ran, run_all};
use batdetect::process::Wording as _;
use batdetect::process::{Versions, process};
use batdetect::refusals::Refusals;
use batdetect::reuse::{self, Run};
use batdetect::tally::Tally;
use batdetect::writable::unwritable_folders;
use batdetect::written::{self, Wording};
use jiff::Timestamp;
use media::decode::Threads;

use crate::arguments::Request;
use crate::errors::report;
use crate::status::{Status, Video};
use crate::texts::{Fixed, Message, Texts};

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
    let stock = match admitted(&request, texts) {
        Ok(admitted) => admitted,
        Err(code) => return code,
    };
    let jobs = stock.jobs();
    let refused = unwritable_folders(&jobs);
    if !refused.is_empty() {
        let (heading, lines) = messages::folders_refused(texts, &refused);
        return report(texts, arguments::rejection(texts, &heading, &lines));
    }
    let status = Arc::new(Status::new(texts, io::stderr().is_terminal(), jobs.len()).redoing_all(request.force));
    let watched = Arc::clone(&status);
    let stopped = jobs.clone();
    let launched = Timestamp::now();
    let refused = stock.refused.clone();
    let watching = ctrlc::set_handler(move || {
        settle(&stopped, &refused, launched, watched.texts());
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
    run(stock, &jobs, &status, &request, launched)
}

fn admitted(request: &Request, texts: Texts) -> Result<Stock, ExitCode> {
    let (jobs, mut refused) = match plan(&request.inputs, request.out_dir.as_deref()) {
        Ok(jobs) => (jobs, Vec::new()),
        Err(jobs::Error::Refused { refused, valid }) => (valid, refused),
        Err(error) => {
            let (heading, lines) = messages::inputs_refused(texts, &error);
            return Err(report(texts, arguments::rejection(texts, &heading, &lines)));
        }
    };
    let run = Run {
        options: &request.options,
        version: messages::version(),
        language: texts.language(),
    };
    let mut stock = take_stock(jobs, |job| if request.force { None } else { reuse::earlier(job, run) });
    refused.append(&mut stock.refused);
    stock.refused = refused;
    if stock.refused.is_empty() {
        return Ok(stock);
    }
    let asking = Asking {
        skip_refused: request.skip_refused,
        terminal: io::stdin().is_terminal(),
    };
    if stock.listed.is_empty() || !asking.shows_the_refusals_first() {
        let error = jobs::Error::Refused {
            valid: stock.jobs(),
            refused: stock.refused,
        };
        let (heading, lines) = messages::inputs_refused(texts, &error);
        return Err(report(texts, arguments::rejection(texts, &heading, &lines)));
    }
    let valid = stock.listed.len();
    let (heading, lines) = messages::inputs_skipped(texts, &stock.refused);
    if errors::show(texts, arguments::rejection(texts, &heading, &lines)).is_err() {
        return Err(ExitCode::FAILURE);
    }
    let question = texts.say(&Message::SkipRefusedQuestion { count: valid });
    match asking.choose(&question, &mut io::stdin().lock(), &mut io::stderr().lock()) {
        Ok(Choice::Process) => Ok(stock),
        Ok(Choice::Refuse) | Err(_) => {
            let declined = Message::Fixed(Fixed::SkipRefusedDeclined);
            Err(report(texts, arguments::rejection(texts, &declined, &[])))
        }
    }
}

fn run(stock: Stock, jobs: &[Job], status: &Status, request: &Request, launched: Timestamp) -> ExitCode {
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
            .filter_map(|root| written::write_page(root, jobs, 0, wording).err()),
    );
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
            unwritten.extend(refusals.note());
            unwritten.extend(written::write_page(&written::root_of(job), jobs, rank.position, wording).err());
            let video = Video { status, rank, job };
            video.tell(&mut io::stderr().lock(), result, options)
        },
    );
    let Ok(rows) = rows else {
        settle(jobs, &stock.refused, launched, texts);
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

fn settle(jobs: &[Job], refused: &[Refused], launched: Timestamp, texts: Texts) {
    let wording = Wording {
        labels: &texts,
        language: texts.language(),
    };
    let roots = written::roots(jobs);
    Refusals::new(roots.clone(), refused, launched).note();
    for root in roots {
        match written::write_page(&root, jobs, jobs.len(), wording) {
            Ok(()) | Err(_) => {}
        }
    }
}
