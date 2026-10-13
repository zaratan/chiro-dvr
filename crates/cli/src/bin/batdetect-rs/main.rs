mod arguments;
mod messages;

use std::io::{self, Write};
use std::process::ExitCode;

use batdetect::jobs::{Job, collect_videos, plan_jobs};
use batdetect::language::system_language;
use batdetect::options::Options;
use batdetect::outcome::{Failure, all_succeeded, run_all};
use batdetect::process::{self, Processed, Versions, process};
use media::decode::Threads;
use types::language::Language;

fn main() -> ExitCode {
    let request = match arguments::parse(std::env::args_os()) {
        Ok(request) => request,
        Err(error) => return report(&error),
    };
    let jobs = match collect_videos(&request.inputs).and_then(|videos| plan_jobs(&videos, &request.out_dir)) {
        Ok(jobs) if jobs.is_empty() => return report(&arguments::refusal(messages::NO_VIDEO)),
        Ok(jobs) => jobs,
        Err(error) => return report(&arguments::refusal(&messages::causes(&error))),
    };
    run(&jobs, system_language(), &request.options)
}

fn run(jobs: &[Job], language: Language, options: &Options) -> ExitCode {
    let ffmpeg = media::library::version();
    let versions = Versions {
        program: messages::version(),
        ffmpeg: &ffmpeg,
    };
    let labels = messages::SummaryLabels(language);
    let outcomes = run_all(
        jobs,
        |job| process(job, language, &labels, Threads::Automatic, versions, options),
        |job, result| tell(language, job, result),
    );
    match outcomes {
        Ok(outcomes) if all_succeeded(&outcomes) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

fn tell(language: Language, job: &Job, result: &Result<Processed, Failure>) -> io::Result<()> {
    match result {
        Ok(processed) => {
            let mut stderr = io::stderr().lock();
            for redone in &processed.rendered.redone {
                writeln!(
                    stderr,
                    "{}",
                    messages::redone(language, job, &redone.paths, &redone.cause)
                )?;
            }
            writeln!(
                io::stdout().lock(),
                "{}",
                messages::processed(language, job, processed.tracks.len())
            )
        }
        Err(Failure::Error(process::Error::TooManyTracks {
            count,
            limit,
            annotated,
        })) => writeln!(
            io::stderr().lock(),
            "{}",
            messages::too_many_tracks(language, job, *count, *limit, *annotated)
        ),
        Err(Failure::Error(error)) => writeln!(io::stderr().lock(), "{}", messages::failed(job, error)),
        Err(Failure::Panic(message)) => writeln!(io::stderr().lock(), "{}", messages::internal_error(job, message)),
    }
}

fn report(error: &clap::Error) -> ExitCode {
    match (error.print(), u8::try_from(error.exit_code())) {
        (Ok(()), Ok(code)) => ExitCode::from(code),
        _ => ExitCode::FAILURE,
    }
}
