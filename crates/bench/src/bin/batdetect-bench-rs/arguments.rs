use std::ffi::OsString;
use std::num::{NonZeroU32, NonZeroUsize};
use std::path::PathBuf;

use bench::config::{DEFAULT_AMPLITUDES, DEFAULT_PURITY, DEFAULT_RADIUS, DEFAULT_SIGMAS, MatchConfig, classes};
use bench::metrics::Judging;
use bench::synthetic::Motion;
use bench::synthetic::sampling::Sampling;
use clap::builder::PossibleValuesParser;
use clap::error::ErrorKind;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use engine::config::DetectConfig;
use engine::settings::{DetectSettings, Region};
use tracking::settings::{StabilitySettings, TrackSettings};
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

use crate::messages;

#[cfg(test)]
mod tests;

const DEFAULT_OUT_DIR: &str = "out/bench";
const DEFAULT_PER_CLASS: usize = 30;
const DEFAULT_SEED: u64 = 1;
const SAMPLING: [&str; 5] = ["per-class", "amplitudes", "sigmas", "motions", "seed"];
const DETECTION: [&str; 10] = [
    "threshold",
    "min-area",
    "max-area",
    "bg-window",
    "bg-step",
    "osd-region",
    "merge-radius",
    "work-width",
    "noise-factor",
    "target-sigma",
];

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Source {
    Drawn { video: PathBuf, sampling: Sampling },
    Targets { video: PathBuf, file: PathBuf },
    Against { video: PathBuf, folder: PathBuf },
    Replay { folder: PathBuf },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Request {
    pub(crate) source: Source,
    pub(crate) out_dir: PathBuf,
    pub(crate) detect: DetectConfig,
    pub(crate) settings: Judging,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", messages::REGION_SHAPE)]
pub(crate) struct RegionShape;

fn region(text: &str) -> Result<Region, RegionShape> {
    let bounds: Vec<f64> = text
        .split(',')
        .map(|bound| bound.trim().parse().ok())
        .collect::<Option<_>>()
        .ok_or(RegionShape)?;
    match bounds[..] {
        [x0, y0, x1, y1] => Ok(Region { x0, y0, x1, y1 }),
        _ => Err(RegionShape),
    }
}

fn number(id: &'static str, value: &impl ToString, help: &'static str) -> Arg {
    Arg::new(id)
        .long(id)
        .allow_negative_numbers(true)
        .default_value(value.to_string())
        .help(help)
}

fn list(id: &'static str, values: &[f64], help: &'static str) -> Arg {
    Arg::new(id)
        .long(id)
        .num_args(1..)
        .allow_negative_numbers(true)
        .value_parser(value_parser!(f64))
        .default_values(values.iter().map(ToString::to_string))
        .help(help)
}

fn bench_arguments() -> Vec<Arg> {
    vec![
        Arg::new("video")
            .value_name("VIDEO")
            .value_parser(value_parser!(PathBuf))
            .required_unless_present("replay")
            .help(messages::VIDEO_HELP),
        Arg::new("out-dir")
            .short('o')
            .long("out-dir")
            .value_name("FOLDER")
            .value_parser(value_parser!(PathBuf))
            .default_value(DEFAULT_OUT_DIR)
            .help(messages::OUT_DIR_HELP),
        number("per-class", &DEFAULT_PER_CLASS, messages::PER_CLASS_HELP).value_parser(value_parser!(NonZeroUsize)),
        list("amplitudes", &DEFAULT_AMPLITUDES, messages::AMPLITUDES_HELP),
        list("sigmas", &DEFAULT_SIGMAS, messages::SIGMAS_HELP),
        Arg::new("motions")
            .long("motions")
            .num_args(1..)
            .value_parser(PossibleValuesParser::new(Motion::ALL.map(Motion::name)))
            .default_value(Motion::Pass.name())
            .help(messages::MOTIONS_HELP),
        number("seed", &DEFAULT_SEED, messages::SEED_HELP).value_parser(value_parser!(u64)),
        number("match-radius", &DEFAULT_RADIUS, messages::MATCH_RADIUS_HELP).value_parser(value_parser!(f64)),
        Arg::new("targets")
            .long("targets")
            .value_name("FILE")
            .value_parser(value_parser!(PathBuf))
            .conflicts_with_all(SAMPLING)
            .conflicts_with("against")
            .help(messages::TARGETS_HELP),
        Arg::new("against")
            .long("against")
            .value_name("FOLDER")
            .value_parser(value_parser!(PathBuf))
            .conflicts_with_all(SAMPLING)
            .help(messages::AGAINST_HELP),
        Arg::new("replay")
            .long("replay")
            .value_name("FOLDER")
            .value_parser(value_parser!(PathBuf))
            .conflicts_with_all(["video", "targets", "against"])
            .conflicts_with_all(SAMPLING)
            .conflicts_with_all(DETECTION)
            .help(messages::REPLAY_HELP),
    ]
}

fn detection_arguments() -> Vec<Arg> {
    let detect = DetectSettings::default();
    let stability = StabilitySettings::from(StabilityConfig::default());
    vec![
        number("threshold", &detect.threshold, messages::THRESHOLD_HELP).value_parser(value_parser!(f64)),
        number("min-area", &detect.min_area, messages::MIN_AREA_HELP).value_parser(value_parser!(f64)),
        number("max-area", &detect.max_area, messages::MAX_AREA_HELP).value_parser(value_parser!(f64)),
        number("bg-window", &detect.bg_window_s, messages::BG_WINDOW_HELP).value_parser(value_parser!(f64)),
        number("bg-step", &detect.bg_step, messages::BG_STEP_HELP).value_parser(value_parser!(usize)),
        Arg::new("osd-region")
            .long("osd-region")
            .value_name("X0,Y0,X1,Y1")
            .action(ArgAction::Append)
            .value_parser(region)
            .help(messages::OSD_REGION_HELP),
        number("merge-radius", &detect.merge_radius, messages::MERGE_RADIUS_HELP).value_parser(value_parser!(f64)),
        number("work-width", &detect.work_width, messages::WORK_WIDTH_HELP).value_parser(value_parser!(u32)),
        number("noise-factor", &detect.noise_factor, messages::NOISE_FACTOR_HELP).value_parser(value_parser!(f64)),
        number("target-sigma", &detect.target_sigma, messages::TARGET_SIGMA_HELP).value_parser(value_parser!(f64)),
        number("max-blobs", &stability.max_blobs, messages::MAX_BLOBS_HELP).value_parser(value_parser!(u32)),
        number("unstable-pad", &stability.pad_s, messages::UNSTABLE_PAD_HELP).value_parser(value_parser!(f64)),
    ]
}

fn tracking_arguments() -> Vec<Arg> {
    let track = TrackSettings::from(TrackConfig::default());
    vec![
        number("max-jump", &track.max_jump, messages::MAX_JUMP_HELP).value_parser(value_parser!(f64)),
        number("max-gap", &track.max_gap, messages::MAX_GAP_HELP).value_parser(value_parser!(NonZeroUsize)),
        number("min-hits", &track.min_hits, messages::MIN_HITS_HELP).value_parser(value_parser!(NonZeroUsize)),
        number("min-travel", &track.min_travel, messages::MIN_TRAVEL_HELP).value_parser(value_parser!(f64)),
        number("twin-distance", &track.twin_distance, messages::TWIN_DISTANCE_HELP).value_parser(value_parser!(f64)),
        number(
            "max-median-turn",
            &track.max_median_turn,
            messages::MAX_MEDIAN_TURN_HELP,
        )
        .value_parser(value_parser!(f64)),
    ]
}

pub(crate) fn command() -> Command {
    Command::new("batdetect-bench-rs")
        .version(messages::version())
        .about(messages::ABOUT)
        .arg_required_else_help(true)
        .next_help_heading(messages::BENCH_HEADING)
        .args(bench_arguments())
        .next_help_heading(messages::DETECTION_HEADING)
        .args(detection_arguments())
        .next_help_heading(messages::TRACKING_HEADING)
        .args(tracking_arguments())
}

pub(crate) fn refusal(message: &str) -> clap::Error {
    command().error(ErrorKind::ValueValidation, message)
}

fn one<T: Clone + Send + Sync + 'static>(matches: &mut ArgMatches, id: &str) -> Result<T, clap::Error> {
    matches.remove_one::<T>(id).ok_or_else(|| refusal(id))
}

fn many<T: Clone + Send + Sync + 'static>(matches: &mut ArgMatches, id: &str) -> Vec<T> {
    matches.remove_many::<T>(id).map(Iterator::collect).unwrap_or_default()
}

fn detection(matches: &mut ArgMatches) -> Result<DetectConfig, clap::Error> {
    let settings = DetectSettings {
        threshold: one(matches, "threshold")?,
        min_area: one(matches, "min-area")?,
        max_area: one(matches, "max-area")?,
        bg_window_s: one(matches, "bg-window")?,
        bg_step: one(matches, "bg-step")?,
        osd_regions: many(matches, "osd-region"),
        merge_radius: one(matches, "merge-radius")?,
        work_width: one(matches, "work-width")?,
        noise_factor: one(matches, "noise-factor")?,
        target_sigma: one(matches, "target-sigma")?,
    };
    DetectConfig::try_from(&settings).map_err(|error| refusal(&messages::detection_refused(&error)))
}

fn settings(matches: &mut ArgMatches) -> Result<Judging, clap::Error> {
    let tracking_refused = |error| refusal(&messages::tracking_refused(&error));
    let stability = StabilityConfig::new(
        NonZeroU32::new(one(matches, "max-blobs")?),
        one(matches, "unstable-pad")?,
    )
    .map_err(tracking_refused)?;
    let twin_distance: f64 = one(matches, "twin-distance")?;
    let track = TrackConfig::new(
        one(matches, "max-jump")?,
        one(matches, "max-gap")?,
        one(matches, "min-hits")?,
        one(matches, "min-travel")?,
        (twin_distance != 0.0).then_some(twin_distance),
        one(matches, "max-median-turn")?,
    )
    .map_err(tracking_refused)?;
    let matching = MatchConfig::new(one(matches, "match-radius")?, DEFAULT_PURITY)
        .map_err(|error| refusal(&messages::match_refused(&error)))?;
    Ok(Judging {
        track,
        stability,
        matching,
    })
}

fn sampling(matches: &mut ArgMatches, matching: MatchConfig) -> Result<Sampling, clap::Error> {
    let motions: Vec<Motion> = many::<String>(matches, "motions")
        .iter()
        .filter_map(|name| name.parse().ok())
        .collect();
    let classes = classes(&motions, &many(matches, "amplitudes"), &many(matches, "sigmas"))
        .map_err(|error| refusal(&messages::class_refused(&error)))?;
    Ok(Sampling {
        seed: one(matches, "seed")?,
        classes,
        per_class: one(matches, "per-class")?,
        min_distance: 3.0 * matching.radius(),
    })
}

fn source(matches: &mut ArgMatches, matching: MatchConfig) -> Result<Source, clap::Error> {
    if let Some(folder) = matches.remove_one::<PathBuf>("replay") {
        return Ok(Source::Replay { folder });
    }
    let video = matches
        .remove_one::<PathBuf>("video")
        .ok_or_else(|| refusal(messages::NO_VIDEO))?;
    if let Some(file) = matches.remove_one::<PathBuf>("targets") {
        return Ok(Source::Targets { video, file });
    }
    if let Some(folder) = matches.remove_one::<PathBuf>("against") {
        return Ok(Source::Against { video, folder });
    }
    Ok(Source::Drawn {
        video,
        sampling: sampling(matches, matching)?,
    })
}

pub(crate) fn parse(arguments: impl IntoIterator<Item = impl Into<OsString> + Clone>) -> Result<Request, clap::Error> {
    let mut matches = command().try_get_matches_from(arguments)?;
    let settings = settings(&mut matches)?;
    let detect = detection(&mut matches)?;
    Ok(Request {
        source: source(&mut matches, settings.matching)?,
        out_dir: one(&mut matches, "out-dir")?,
        detect,
        settings,
    })
}
