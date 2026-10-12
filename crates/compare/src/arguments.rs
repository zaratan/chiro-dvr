use std::ffi::OsString;
use std::path::PathBuf;

use clap::builder::{EnumValueParser, PossibleValue};
use clap::error::ErrorKind;
use clap::{Arg, Command, ValueEnum, value_parser};

use crate::case::Target;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub target: Target,
    pub frames: Option<PathBuf>,
}

impl ValueEnum for Target {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Extract, Self::Whole, Self::Settings, Self::Recorded]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Extract => PossibleValue::new("extract").help("the extract of the 092, in the fixtures"),
            Self::Whole => PossibleValue::new("092").help("the whole 092, in the fixtures"),
            Self::Settings => PossibleValue::new("settings")
                .help("the extract and its 720×540 reduction, recorded by the v0.1.1 under other detection settings"),
            Self::Recorded => PossibleValue::new("local")
                .help("the 26 recorded videos, by BATDETECT_REFERENCE_DIR and BATDETECT_VIDEOS_DIR"),
        })
    }
}

pub fn command() -> Command {
    Command::new("compare")
        .about("Runs the detection and compares it with the v0.1.1 reference")
        .arg(
            Arg::new("videos")
                .required(true)
                .value_parser(EnumValueParser::<Target>::new()),
        )
        .arg(
            Arg::new("frames")
                .long("frames")
                .value_name("FOLDER")
                .value_parser(value_parser!(PathBuf))
                .help("writes the detections of each video to FOLDER/<video>/frames.jsonl"),
        )
}

pub fn parse(arguments: impl IntoIterator<Item = impl Into<OsString> + Clone>) -> Result<Request, clap::Error> {
    let mut matches = command().try_get_matches_from(arguments)?;
    let target = matches
        .remove_one::<Target>("videos")
        .ok_or_else(|| command().error(ErrorKind::MissingRequiredArgument, "the videos to compare"))?;
    Ok(Request {
        target,
        frames: matches.remove_one::<PathBuf>("frames"),
    })
}
