use std::ffi::OsString;
use std::num::{NonZeroU32, NonZeroUsize};
use std::path::PathBuf;

use batdetect::options::Options;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::builder::{PossibleValuesParser, StyledStr, TypedValueParser};
use clap::error::ErrorKind;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use engine::config::DetectConfig;
use engine::settings::{DetectSettings, Region};
use render::config::{RenderConfig, Zoom, ZoomConfig};
use tracking::settings::StabilitySettings;
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

use crate::messages;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Request {
    pub(crate) inputs: Vec<PathBuf>,
    pub(crate) out_dir: PathBuf,
    pub(crate) options: Options,
}

fn number(id: &'static str, value: &impl ToString, name: &'static str, help: &'static str) -> Arg {
    let value = value.to_string();
    Arg::new(id)
        .long(id)
        .value_name(name)
        .allow_negative_numbers(true)
        .hide_default_value(true)
        .help(with_default(help, &value))
        .default_value(value)
}

fn with_default(help: &str, value: &str) -> StyledStr {
    let valid = STYLES.get_valid();
    StyledStr::from(format!("{help}\n{}{valid}{value}{valid:#}]", messages::DEFAULT_OPENING))
}

fn zoom_help() -> StyledStr {
    let valid = STYLES.get_valid();
    let listed: Vec<String> = Zoom::CHOICES
        .map(Zoom::name)
        .iter()
        .map(|name| format!("{valid}{name}{valid:#}"))
        .collect();
    let mut help = with_default(messages::ZOOM_HELP, Zoom::Auto.name());
    help.push_str(&format!("\n{}{}]", messages::POSSIBLE_OPENING, listed.join(", ")));
    help
}

fn essential_arguments() -> Vec<Arg> {
    vec![
        Arg::new("inputs")
            .value_name("VIDEO")
            .required(true)
            .action(ArgAction::Append)
            .value_parser(value_parser!(PathBuf))
            .help(messages::INPUTS_HELP),
        Arg::new("out-dir")
            .short('o')
            .long("out-dir")
            .value_name("FOLDER")
            .required(true)
            .value_parser(value_parser!(PathBuf))
            .help(messages::OUT_DIR_HELP),
        Arg::new("zoom")
            .long("zoom")
            .value_name("WHICH")
            .default_value(Zoom::Auto.name())
            .hide_default_value(true)
            .hide_possible_values(true)
            .value_parser(
                PossibleValuesParser::new(Zoom::CHOICES.map(Zoom::name)).try_map(|name| {
                    Zoom::CHOICES
                        .into_iter()
                        .find(|choice| choice.name() == name)
                        .ok_or("unknown zoom")
                }),
            )
            .help(zoom_help()),
    ]
}

fn useful_arguments(level: Level) -> Vec<Arg> {
    let render = RenderConfig::default();
    vec![
        Arg::new("annotated")
            .long("annotated")
            .action(ArgAction::SetTrue)
            .help(messages::ANNOTATED_HELP),
        number("max-tracks", &render.max_tracks(), "COUNT", messages::MAX_TRACKS_HELP)
            .value_parser(value_parser!(usize)),
        number(
            "clip-margin",
            &render.clip_margin_s(),
            "SECONDS",
            messages::CLIP_MARGIN_HELP,
        )
        .value_parser(value_parser!(f64)),
        Arg::new("osd-region")
            .long("osd-region")
            .value_name("X0,Y0,X1,Y1")
            .action(ArgAction::Append)
            .value_parser(value_parser!(Region))
            .help(messages::OSD_REGION_HELP),
    ]
    .into_iter()
    .map(|argument| argument.hide(level == Level::Essential))
    .collect()
}

fn advanced_arguments(level: Level) -> Vec<Arg> {
    let detect = DetectSettings::default();
    let track = TrackConfig::default();
    let stability = StabilitySettings::from(StabilityConfig::default());
    vec![
        number("threshold", &detect.threshold, "LEVEL", messages::THRESHOLD_HELP).value_parser(value_parser!(f64)),
        number("min-hits", &track.min_hits(), "COUNT", messages::MIN_HITS_HELP)
            .value_parser(value_parser!(NonZeroUsize)),
        number("max-blobs", &stability.max_blobs, "COUNT", messages::MAX_BLOBS_HELP).value_parser(value_parser!(u32)),
        number("work-width", &detect.work_width, "PIXELS", messages::WORK_WIDTH_HELP).value_parser(value_parser!(u32)),
    ]
    .into_iter()
    .map(|argument| argument.hide(level == Level::Essential))
    .collect()
}

fn level(arguments: &[OsString]) -> Level {
    let full = arguments
        .iter()
        .skip(1)
        .take_while(|argument| *argument != "--")
        .any(|argument| argument == "--help");
    if full { Level::Full } else { Level::Essential }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Level {
    Essential,
    Full,
}

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Blue.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Blue.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Yellow.on_default())
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .context_value(AnsiColor::Green.on_default());

fn command(level: Level) -> Command {
    Command::new("batdetect-rs")
        .version(messages::version())
        .long_version(messages::long_version(
            &media::library::version(),
            media::library::license(),
        ))
        .about(messages::ABOUT)
        .styles(STYLES)
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .args(essential_arguments())
        .args(useful_arguments(level))
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::HelpShort)
                .help(messages::HELP_HELP),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::Version)
                .help(messages::VERSION_HELP),
        )
        .next_help_heading(messages::DETECTION_HEADING)
        .args(advanced_arguments(level))
}

pub(crate) fn parse(arguments: impl IntoIterator<Item = impl Into<OsString>>) -> Result<Request, clap::Error> {
    let arguments: Vec<OsString> = arguments.into_iter().map(Into::into).collect();
    let mut matches = command(level(&arguments)).try_get_matches_from(arguments)?;
    let inputs = many(&mut matches, "inputs");
    let out_dir = one(&mut matches, "out-dir")?;
    let options = options(&mut matches)?;
    Ok(Request {
        inputs,
        out_dir,
        options,
    })
}

fn options(matches: &mut ArgMatches) -> Result<Options, clap::Error> {
    let defaults = DetectSettings::default();
    let settings = DetectSettings {
        threshold: one(matches, "threshold")?,
        work_width: one(matches, "work-width")?,
        osd_regions: many(matches, "osd-region"),
        ..defaults
    };
    let detect = DetectConfig::try_from(&settings).map_err(|error| refusal(&messages::detection_refused(&error)))?;
    let default_track = TrackConfig::default();
    let track = TrackConfig::new(
        default_track.max_jump(),
        default_track.max_gap(),
        one(matches, "min-hits")?,
        default_track.min_travel(),
        default_track.twin_distance(),
        default_track.max_median_turn(),
    )
    .map_err(|error| refusal(&error.to_string()))?;
    let stability = StabilityConfig::new(
        NonZeroU32::new(one(matches, "max-blobs")?),
        StabilityConfig::default().pad_s(),
    )
    .map_err(|error| refusal(&error.to_string()))?;
    let default_render = RenderConfig::default();
    let choice = one::<Zoom>(matches, "zoom")?;
    let render = RenderConfig::new(
        default_render.box_pad(),
        default_render.trail_s(),
        one(matches, "clip-margin")?,
        one(matches, "max-tracks")?,
    )
    .map_err(|error| refusal(&messages::render_refused(&error)))?
    .with_zoom(ZoomConfig::choosing(choice));
    Ok(Options {
        detect,
        track,
        stability,
        render,
        annotated: matches.get_flag("annotated"),
    })
}

fn one<T: Clone + Send + Sync + 'static>(matches: &mut ArgMatches, id: &str) -> Result<T, clap::Error> {
    matches.remove_one::<T>(id).ok_or_else(|| refusal(id))
}

fn many<T: Clone + Send + Sync + 'static>(matches: &mut ArgMatches, id: &str) -> Vec<T> {
    matches.remove_many::<T>(id).map(Iterator::collect).unwrap_or_default()
}

pub(crate) fn refusal(message: &str) -> clap::Error {
    command(Level::Essential).error(ErrorKind::ValueValidation, message)
}
