use std::ffi::OsString;
use std::path::PathBuf;

use clap::builder::{EnumValueParser, PossibleValue, PossibleValuesParser, TypedValueParser};
use clap::error::ErrorKind;
use clap::{Arg, Command, ValueEnum, value_parser};

use reference::location::Reference;

use crate::case::Target;
use crate::comparison::Folders;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub target: Target,
    pub reference: Reference,
    pub folders: Folders,
}

const V020: &str = "v0.2.0";
const V011: &str = "v0.1.1";

impl ValueEnum for Target {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Extract,
            Self::Whole,
            Self::WholeAt1440,
            Self::Settings,
            Self::Recorded,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Extract => PossibleValue::new("extract").help("the extract of the 092, in the fixtures"),
            Self::Whole => PossibleValue::new("092").help("the whole 092, in the fixtures"),
            Self::WholeAt1440 => {
                PossibleValue::new("092-1440").help("the whole 092 worked at 1440 pixels, in the fixtures")
            }
            Self::Settings => PossibleValue::new("settings")
                .help("the extract and its 720×540 reduction, recorded under other detection settings"),
            Self::Recorded => PossibleValue::new("local").help(
                "the 26 recorded videos, by BATDETECT_VIDEOS_DIR and BATDETECT_REFERENCE_V020_DIR or BATDETECT_REFERENCE_DIR",
            ),
        })
    }
}

pub fn command() -> Command {
    Command::new("compare")
        .about("Runs the detection and compares it with a reference")
        .arg(
            Arg::new("videos")
                .required(true)
                .value_parser(EnumValueParser::<Target>::new()),
        )
        .arg(
            Arg::new("reference")
                .long("reference")
                .value_name("RELEASE")
                .default_value(V020)
                .value_parser(PossibleValuesParser::new([V020, V011]).try_map(|release| {
                    reference(&release).ok_or_else(|| format!("no reference {release}"))
                }))
                .help("the reference to compare with: the v0.2.0 recorded by this code, or the v0.1.1 it was rewritten from"),
        )
        .arg(
            Arg::new("frames")
                .long("frames")
                .value_name("FOLDER")
                .value_parser(value_parser!(PathBuf))
                .help("writes the detections of each video to FOLDER/<video>/frames.jsonl"),
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

pub fn parse(arguments: impl IntoIterator<Item = impl Into<OsString> + Clone>) -> Result<Request, clap::Error> {
    let mut matches = command().try_get_matches_from(arguments)?;
    let target = matches
        .remove_one::<Target>("videos")
        .ok_or_else(|| command().error(ErrorKind::MissingRequiredArgument, "the videos to compare"))?;
    Ok(Request {
        target,
        reference: matches
            .remove_one::<Reference>("reference")
            .ok_or_else(|| command().error(ErrorKind::MissingRequiredArgument, "the reference to compare with"))?,
        folders: Folders {
            frames: matches.remove_one::<PathBuf>("frames"),
            same_as: matches.remove_one::<PathBuf>("same-as"),
        },
    })
}

fn reference(release: &str) -> Option<Reference> {
    match release {
        V020 => Some(Reference::V020),
        V011 => Some(Reference::V011),
        _ => None,
    }
}
