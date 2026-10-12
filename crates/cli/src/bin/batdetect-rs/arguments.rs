use std::ffi::OsString;
use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{Arg, ArgAction, Command, value_parser};

use crate::messages;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Request {
    pub(crate) inputs: Vec<PathBuf>,
    pub(crate) output: PathBuf,
}

fn command() -> Command {
    Command::new("batdetect-rs")
        .version(messages::version())
        .long_version(messages::long_version(
            &media::library::version(),
            media::library::license(),
        ))
        .about(messages::ABOUT)
        .arg_required_else_help(true)
        .arg(
            Arg::new("inputs")
                .value_name("VIDEO")
                .required(true)
                .action(ArgAction::Append)
                .value_parser(value_parser!(PathBuf))
                .help(messages::INPUTS_HELP),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FOLDER")
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .help(messages::OUTPUT_HELP),
        )
}

pub(crate) fn parse(arguments: impl IntoIterator<Item = impl Into<OsString> + Clone>) -> Result<Request, clap::Error> {
    let mut matches = command().try_get_matches_from(arguments)?;
    let inputs = matches
        .remove_many::<PathBuf>("inputs")
        .map(Iterator::collect)
        .unwrap_or_default();
    let output = matches
        .remove_one::<PathBuf>("output")
        .ok_or_else(|| refusal(messages::NO_OUTPUT))?;
    Ok(Request { inputs, output })
}

pub(crate) fn refusal(message: &str) -> clap::Error {
    command().error(ErrorKind::ValueValidation, message)
}
