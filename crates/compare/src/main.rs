use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use compare::arguments::{Request, parse};
use compare::comparison::{VIDEOS_AT_ONCE, compare_all};
use compare::report::{write_failure, write_report};

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
    let mut comparisons = Vec::new();
    let mut failed = false;
    for (case, answer) in cases
        .iter()
        .zip(compare_all(&cases, VIDEOS_AT_ONCE, request.frames.as_deref()))
    {
        match answer {
            Ok(comparison) => comparisons.push(comparison),
            Err(error) => {
                failed = true;
                if write_failure(case.recording(), &error, &mut stderr).is_err() {
                    break;
                }
            }
        }
    }
    let written = write_report(&comparisons, &mut io::stdout().lock());
    let held = comparisons.iter().all(|comparison| comparison.detections.holds());
    if written.is_ok() && held && !failed && !comparisons.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
