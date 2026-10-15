mod arguments;
mod execute;
mod messages;

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use crate::arguments::{Request, Source};
use crate::execute::execute;

fn main() -> ExitCode {
    match arguments::parse(std::env::args_os()) {
        Ok(request) => run(&request),
        Err(error) => report(&error),
    }
}

fn subject(request: &Request) -> &Path {
    match &request.source {
        Source::Drawn { video, .. } | Source::Targets { video, .. } | Source::Against { video, .. } => video,
        Source::Replay { folder } => folder,
    }
}

fn run(request: &Request) -> ExitCode {
    let (written, code) = match execute(request) {
        Ok(outcome) => {
            let mut text = format!("{}\n{}\n", outcome.table, outcome.report.display());
            if let Some(guard) = &outcome.guard {
                text.push_str(guard);
                text.push('\n');
            }
            let code = if outcome.failed {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            };
            let mut stdout = io::stdout().lock();
            (stdout.write_all(text.as_bytes()).and_then(|()| stdout.flush()), code)
        }
        Err(error) => {
            let mut stderr = io::stderr().lock();
            (
                writeln!(stderr, "{}", messages::failed(subject(request), &error)),
                ExitCode::FAILURE,
            )
        }
    };
    if written.is_err() { ExitCode::FAILURE } else { code }
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
