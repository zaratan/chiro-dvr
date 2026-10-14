mod arguments;
mod errors;
mod messages;
mod numbers;
mod texts;

use std::io::{self, Write};
use std::process::ExitCode;

use batdetect::jobs::{Job, collect_videos, plan_jobs};
use batdetect::language::system_language;
use batdetect::options::Options;
use batdetect::outcome::{Failure, all_succeeded, run_all};
use batdetect::process::{self, Processed, Versions, process};
use media::decode::Threads;

use crate::errors::report;
use crate::texts::{Fixed, Message, Texts};

fn main() -> ExitCode {
    let texts = Texts::new(system_language());
    let request = match arguments::parse(std::env::args_os(), texts) {
        Ok(request) => request,
        Err(error) => return report(texts, error),
    };
    let jobs = match collect_videos(&request.inputs).and_then(|videos| plan_jobs(&videos, &request.out_dir)) {
        Ok(jobs) if jobs.is_empty() => {
            return report(texts, arguments::refusal(texts, &Message::Fixed(Fixed::NoVideo)));
        }
        Ok(jobs) => jobs,
        Err(error) => return report(texts, arguments::refusal(texts, &messages::videos_refused(&error))),
    };
    run(&jobs, texts, &request.options)
}

fn run(jobs: &[Job], texts: Texts, options: &Options) -> ExitCode {
    let ffmpeg = media::library::version();
    let versions = Versions {
        program: messages::version(),
        ffmpeg: &ffmpeg,
    };
    let outcomes = run_all(
        jobs,
        |job| process(job, &texts, Threads::Automatic, versions, options),
        |job, result| tell(texts, job, result),
    );
    match outcomes {
        Ok(outcomes) if all_succeeded(&outcomes) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

fn tell(texts: Texts, job: &Job, result: &Result<Processed, Failure>) -> io::Result<()> {
    match result {
        Ok(processed) => {
            let mut stderr = io::stderr().lock();
            for kept in &processed.kept {
                writeln!(stderr, "{}", texts.say(&messages::table_kept(job, kept)))?;
            }
            for redone in &processed.rendered.redone {
                writeln!(
                    stderr,
                    "{}",
                    texts.say(&messages::redone(job, &redone.paths, &redone.cause))
                )?;
            }
            writeln!(
                io::stdout().lock(),
                "{}",
                texts.say(&messages::processed(job, processed.tracks.len()))
            )
        }
        Err(Failure::Error(process::Error::TooManyTracks {
            count,
            limit,
            annotated,
        })) => writeln!(
            io::stderr().lock(),
            "{}",
            texts.say(&messages::too_many_tracks(job, *count, *limit, *annotated))
        ),
        Err(Failure::Error(error)) => writeln!(io::stderr().lock(), "{}", texts.say(&messages::failed(job, error))),
        Err(Failure::Panic(message)) => writeln!(
            io::stderr().lock(),
            "{}",
            texts.say(&messages::internal_error(job, message))
        ),
    }
}
