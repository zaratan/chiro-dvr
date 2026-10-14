mod arguments;
mod errors;
mod messages;
mod numbers;
mod status;
mod texts;

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use batdetect::jobs::{Job, collect_videos, plan_jobs};
use batdetect::language::system_language;
use batdetect::options::Options;
use batdetect::outcome::{all_succeeded, run_all};
use batdetect::process::{Versions, process};
use batdetect::writable::unwritable_folders;
use media::decode::Threads;

use crate::errors::report;
use crate::status::{Status, Video};
use crate::texts::{Fixed, Message, Texts};

fn main() -> ExitCode {
    let texts = Texts::new(system_language());
    let request = match arguments::parse(std::env::args_os(), texts) {
        Ok(request) => request,
        Err(error) => return report(texts, error),
    };
    let planned = collect_videos(&request.inputs).and_then(|videos| plan_jobs(&videos, request.out_dir.as_deref()));
    let jobs = match planned {
        Ok(jobs) if jobs.is_empty() => {
            return report(texts, arguments::refusal(texts, &Message::Fixed(Fixed::NoVideo)));
        }
        Ok(jobs) => jobs,
        Err(error) => return report(texts, arguments::refusal(texts, &messages::videos_refused(&error))),
    };
    let refused = unwritable_folders(&jobs);
    if !refused.is_empty() {
        return report(texts, arguments::refusal(texts, &messages::folders_refused(&refused)));
    }
    run(&jobs, texts, &request.options)
}

fn run(jobs: &[Job], texts: Texts, options: &Options) -> ExitCode {
    let ffmpeg = media::library::version();
    let versions = Versions {
        program: messages::version(),
        ffmpeg: &ffmpeg,
    };
    let status = Status::new(texts, io::stderr().is_terminal());
    let outcomes = run_all(
        jobs,
        |job, rank| {
            let video = Video {
                status: &status,
                rank,
                job,
            };
            process(job, &texts, &video, Threads::Automatic, versions, options)
        },
        |job, rank, result| {
            let video = Video {
                status: &status,
                rank,
                job,
            };
            video.tell(&mut io::stderr().lock(), &mut io::stdout().lock(), result)
        },
    );
    match outcomes {
        Ok(outcomes) if all_succeeded(&outcomes) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}
