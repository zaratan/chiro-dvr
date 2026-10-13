mod arguments;
mod messages;

use std::io::{self, Write};
use std::process::ExitCode;

use batdetect::jobs::{Job, collect_videos, plan_jobs};
use batdetect::language::system_language;
use batdetect::process::{self, Versions, process};
use media::decode::Threads;
use types::language::Language;

fn main() -> ExitCode {
    let request = match arguments::parse(std::env::args_os()) {
        Ok(request) => request,
        Err(error) => return report(&error),
    };
    let jobs = match collect_videos(&request.inputs).and_then(|videos| plan_jobs(&videos, &request.output)) {
        Ok(jobs) if jobs.is_empty() => return report(&arguments::refusal(messages::NO_VIDEO)),
        Ok(jobs) => jobs,
        Err(error) => return report(&arguments::refusal(&messages::causes(&error))),
    };
    run(&jobs, system_language())
}

fn run(jobs: &[Job], language: Language) -> ExitCode {
    let ffmpeg = media::library::version();
    let versions = Versions {
        program: messages::version(),
        ffmpeg: &ffmpeg,
    };
    let mut failures = 0_usize;
    for job in jobs {
        let labels = messages::SummaryLabels(language);
        let line = match process(job, language, &labels, Threads::Automatic, versions) {
            Ok(processed) => {
                let mut stderr = io::stderr().lock();
                let redone = processed.rendered.redone.iter().try_for_each(|redone| {
                    writeln!(
                        stderr,
                        "{}",
                        messages::redone(language, job, &redone.paths, &redone.cause)
                    )
                });
                redone.and_then(|()| {
                    writeln!(
                        io::stdout().lock(),
                        "{}",
                        messages::processed(language, job, processed.tracks.len())
                    )
                })
            }
            Err(process::Error::TooManyTracks { count, limit }) => {
                failures += 1;
                writeln!(
                    io::stderr().lock(),
                    "{}",
                    messages::too_many_tracks(language, job, count, limit)
                )
            }
            Err(error) => {
                failures += 1;
                writeln!(io::stderr().lock(), "{}", messages::failed(job, &error))
            }
        };
        if line.is_err() {
            return ExitCode::FAILURE;
        }
    }
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn report(error: &clap::Error) -> ExitCode {
    let text = error.render().to_string();
    let written = if error.use_stderr() {
        let mut stderr = io::stderr().lock();
        stderr.write_all(text.as_bytes()).and_then(|()| stderr.flush())
    } else {
        let mut stdout = io::stdout().lock();
        stdout.write_all(text.as_bytes()).and_then(|()| stdout.flush())
    };
    match (written, u8::try_from(error.exit_code())) {
        (Ok(()), Ok(code)) => ExitCode::from(code),
        _ => ExitCode::FAILURE,
    }
}
