use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use compare::arguments::{Request, parse};
use compare::comparison::{VIDEOS_AT_ONCE, compare_all};
use compare::outcome::Outcome;
use compare::report::{write_failure, write_refusals, write_report};

fn main() -> ExitCode {
    match parse(env::args_os()) {
        Ok(request) => run(&request),
        Err(error) => match (error.print(), u8::try_from(error.exit_code())) {
            (Ok(()), Ok(code)) => ExitCode::from(code),
            _ => ExitCode::FAILURE,
        },
    }
}

fn run(request: &Request) -> ExitCode {
    let mut stderr = io::stderr().lock();
    let cases = match request.target.cases() {
        Ok(cases) => cases,
        Err(error) => {
            return match writeln!(stderr, "{error}") {
                Ok(()) | Err(_) => ExitCode::FAILURE,
            };
        }
    };
    let answers = compare_all(&cases, VIDEOS_AT_ONCE, request.frames.as_deref());
    let outcome = Outcome::of(&cases, answers);
    for (recording, error) in &outcome.failed {
        if write_failure(recording, error, &mut stderr).is_err() {
            break;
        }
    }
    let mut stdout = io::stdout().lock();
    let written =
        write_report(&outcome.compared, &mut stdout).and_then(|()| write_refusals(&outcome.refused, &mut stdout));
    if written.is_ok() && outcome.holds(request.target.tolerates_refusals()) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
