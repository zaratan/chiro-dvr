mod messages;

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let command = clap::Command::new("batdetect-rs")
        .version(messages::version())
        .long_version(messages::long_version(
            &media::library::version(),
            media::library::license(),
        ))
        .arg_required_else_help(true);
    command
        .try_get_matches()
        .map_or_else(|error| report(&error), |_| ExitCode::SUCCESS)
}

fn report(error: &clap::Error) -> ExitCode {
    let text = error.render().to_string();
    let written = if error.use_stderr() {
        io::stderr().lock().write_all(text.as_bytes())
    } else {
        io::stdout().lock().write_all(text.as_bytes())
    };
    match (written, u8::try_from(error.exit_code())) {
        (Ok(()), Ok(code)) => ExitCode::from(code),
        _ => ExitCode::FAILURE,
    }
}
