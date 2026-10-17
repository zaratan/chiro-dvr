use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use compare::comparison::VIDEOS_AT_ONCE;
use compare::provenance::Build;
use compare::record_arguments::{RecordRequest, parse};
use compare::recorder::{Places, holds, protected_folders, record_all, refuse_overlap};
use compare::report::write_failure;
use reference::location::Reference;

const NO_CASE: &str = "no case to record";

fn main() -> ExitCode {
    match parse(env::args_os()) {
        Ok(request) => run(&request),
        Err(error) => match (error.print(), u8::try_from(error.exit_code())) {
            (Ok(()), Ok(code)) => ExitCode::from(code),
            _ => ExitCode::FAILURE,
        },
    }
}

fn run(request: &RecordRequest) -> ExitCode {
    let mut stderr = io::stderr().lock();
    let protected = protected_folders(request.same_as.as_deref());
    let started = Build::of_this_binary()
        .map_err(|refusal| refusal.to_string())
        .and_then(|build| {
            refuse_overlap(&request.out, &protected).map_err(|error| error.to_string())?;
            let cases = request
                .target
                .cases(Reference::V011)
                .map_err(|error| error.to_string())?;
            if cases.is_empty() {
                return Err(NO_CASE.to_owned());
            }
            Ok((build, cases))
        });
    let (build, cases) = match started {
        Ok(started) => started,
        Err(message) => {
            return match writeln!(stderr, "{message}") {
                Ok(()) | Err(_) => ExitCode::FAILURE,
            };
        }
    };
    let places = Places {
        out: &request.out,
        same_as: request.same_as.as_deref(),
    };
    let answers = record_all(&cases, VIDEOS_AT_ONCE, &build, &places);
    let mut stdout = io::stdout().lock();
    for (case, answer) in cases.iter().zip(&answers) {
        let written = match answer {
            Ok(recorded) => recorded.write_line(&mut stdout),
            Err(error) => write_failure(case.recording(), error, &mut stderr),
        };
        if written.is_err() {
            return ExitCode::FAILURE;
        }
    }
    if holds(&answers) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
