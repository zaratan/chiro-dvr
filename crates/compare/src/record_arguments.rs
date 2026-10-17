use std::ffi::OsString;
use std::path::PathBuf;

use clap::builder::EnumValueParser;
use clap::error::ErrorKind;
use clap::{Arg, Command, value_parser};

use crate::case::Target;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordRequest {
    pub target: Target,
    pub out: PathBuf,
    pub same_as: Option<PathBuf>,
}

pub fn command() -> Command {
    Command::new("record")
        .about("Records the detections and tracks of the v0.1.1 cases as a reference of the decoder, from a committed binary")
        .arg(
            Arg::new("videos")
                .required(true)
                .value_parser(EnumValueParser::<Target>::new()),
        )
        .arg(
            Arg::new("out")
                .long("out")
                .required(true)
                .value_name("FOLDER")
                .value_parser(value_parser!(PathBuf))
                .help("writes each recording to FOLDER, in the tree of the v0.1.1 reference"),
        )
        .arg(
            Arg::new("same-as")
                .long("same-as")
                .value_name("FOLDER")
                .value_parser(value_parser!(PathBuf))
                .help(
                    "checks that the detections equal those of FOLDER/<video>/frames.jsonl outside the damaged spans",
                ),
        )
}

pub fn parse(arguments: impl IntoIterator<Item = impl Into<OsString> + Clone>) -> Result<RecordRequest, clap::Error> {
    let mut matches = command().try_get_matches_from(arguments)?;
    let target = matches
        .remove_one::<Target>("videos")
        .ok_or_else(|| command().error(ErrorKind::MissingRequiredArgument, "the videos to record"))?;
    let out = matches
        .remove_one::<PathBuf>("out")
        .ok_or_else(|| command().error(ErrorKind::MissingRequiredArgument, "the folder to record into"))?;
    Ok(RecordRequest {
        target,
        out,
        same_as: matches.remove_one::<PathBuf>("same-as"),
    })
}
