use std::ffi::OsString;
use std::io;
use std::num::{NonZeroU32, NonZeroUsize};
use std::path::PathBuf;

use batdetect::options::Options;
use clap::builder::{PossibleValuesParser, StyledStr, TypedValueParser};
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use engine::config::{DetectConfig, MIN_WORK_WIDTH};
use engine::settings::{Bound, DetectSettings, Region, Setting};
use render::config::{RenderConfig, Zoom, ZoomConfig};
use tracking::settings::StabilitySettings;
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

use crate::errors;
use crate::messages;
use crate::parsing::{Expected, Reader};
use crate::style::{HELP, STYLES};
use crate::texts::{Fixed, Message, Texts};

pub(crate) const PROGRAM: &str = env!("CARGO_BIN_NAME");
pub(crate) const MIN_HITS: &str = "min-hits";
pub(crate) const MAX_BLOBS: &str = "max-blobs";
pub(crate) const OSD_REGION: &str = "osd-region";
const SKIP_REFUSED: &str = "skip-refused";

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Request {
    pub(crate) inputs: Vec<PathBuf>,
    pub(crate) out_dir: Option<PathBuf>,
    pub(crate) options: Options,
    pub(crate) skip_refused: bool,
}

fn number(id: &'static str, value: &impl ToString, name: Fixed, help: Fixed, texts: Texts) -> Arg {
    let value = value.to_string();
    let shown = value
        .parse::<f64>()
        .map_or_else(|_| value.clone(), |number| texts.number(number));
    Arg::new(id)
        .long(id)
        .value_name(texts.text(name))
        .allow_negative_numbers(true)
        .hide_default_value(true)
        .help(with_default(&texts.text(help), &shown, texts))
        .default_value(value)
}

fn with_default(help: &str, value: &str, texts: Texts) -> StyledStr {
    let valid = STYLES.get_valid();
    let value = format!("{valid}{value}{valid:#}");
    StyledStr::from(format!("{help}\n{}", texts.say(&Message::DefaultValue { value })))
}

fn zoom_help(texts: Texts) -> StyledStr {
    let valid = STYLES.get_valid();
    let listed: Vec<String> = Zoom::CHOICES
        .map(Zoom::name)
        .iter()
        .map(|name| format!("{valid}{name}{valid:#}"))
        .collect();
    let mut help = with_default(&texts.text(Fixed::ZoomHelp), Zoom::Auto.name(), texts);
    let values = listed.join(", ");
    help.push_str(&format!("\n{}", texts.say(&Message::PossibleValues { values })));
    help
}

fn essential_arguments(texts: Texts) -> Vec<Arg> {
    vec![
        Arg::new("inputs")
            .value_name(texts.text(Fixed::ValueVideo))
            .required(true)
            .action(ArgAction::Append)
            .value_parser(value_parser!(PathBuf))
            .help_heading(texts.text(Fixed::ArgumentsHeading))
            .help(texts.text(Fixed::InputsHelp)),
        Arg::new("out-dir")
            .short('o')
            .long("out-dir")
            .value_name(texts.text(Fixed::ValueFolder))
            .value_parser(value_parser!(PathBuf))
            .help(with_default(
                &texts.text(Fixed::OutDirHelp),
                &texts.text(Fixed::BesideTheVideos),
                texts,
            )),
        Arg::new("zoom")
            .long("zoom")
            .value_name(texts.text(Fixed::ValueWhich))
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
            .help(zoom_help(texts)),
    ]
}

fn useful_arguments(level: Level, texts: Texts) -> Vec<Arg> {
    let render = RenderConfig::default();
    vec![
        Arg::new("annotated")
            .long("annotated")
            .action(ArgAction::SetTrue)
            .help(texts.text(Fixed::AnnotatedHelp)),
        number(
            "max-tracks",
            &render.max_tracks(),
            Fixed::ValueCount,
            Fixed::MaxTracksHelp,
            texts,
        )
        .value_parser(Reader::<usize>::exact(Expected::Setting(Bound::AtLeast(0)))),
        number(
            "clip-margin",
            &render.clip_margin_s(),
            Fixed::ValueSeconds,
            Fixed::ClipMarginHelp,
            texts,
        )
        .value_parser(Reader::<f64>::decimal(Expected::Seconds)),
        Arg::new(OSD_REGION)
            .long(OSD_REGION)
            .value_name("X0,Y0,X1,Y1")
            .action(ArgAction::Append)
            .allow_hyphen_values(true)
            .value_parser(Reader::<Region>::exact(Expected::Region))
            .help(with_default(
                &texts.text(Fixed::OsdRegionHelp),
                &texts.text(Fixed::NoRegion),
                texts,
            )),
        Arg::new(SKIP_REFUSED)
            .long(SKIP_REFUSED)
            .action(ArgAction::SetTrue)
            .help(texts.text(Fixed::SkipRefusedHelp)),
    ]
    .into_iter()
    .map(|argument| argument.hide(level == Level::Essential))
    .collect()
}

fn advanced_arguments(level: Level, texts: Texts) -> Vec<Arg> {
    let detect = DetectSettings::default();
    let track = TrackConfig::default();
    let stability = StabilitySettings::from(StabilityConfig::default());
    vec![
        number(
            Setting::Threshold.flag(),
            &detect.threshold,
            Fixed::ValueLevel,
            Fixed::ThresholdHelp,
            texts,
        )
        .value_parser(Reader::<f64>::decimal(Expected::Setting(Bound::Positive))),
        number(
            MIN_HITS,
            &track.min_hits(),
            Fixed::ValueCount,
            Fixed::MinHitsHelp,
            texts,
        )
        .value_parser(Reader::<NonZeroUsize>::exact(Expected::Setting(Bound::AtLeast(1)))),
        number(
            MAX_BLOBS,
            &stability.max_blobs,
            Fixed::ValueCount,
            Fixed::MaxBlobsHelp,
            texts,
        )
        .value_parser(Reader::<u32>::exact(Expected::Setting(Bound::AtLeast(0)))),
        number(
            Setting::WorkWidth.flag(),
            &detect.work_width,
            Fixed::ValuePixels,
            Fixed::WorkWidthHelp,
            texts,
        )
        .value_parser(Reader::<u32>::exact(Expected::Setting(Bound::AtLeast(MIN_WORK_WIDTH)))),
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
        .any(|argument| argument.to_str().and_then(|text| text.strip_prefix("--")) == Some(HELP));
    if full { Level::Full } else { Level::Essential }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Level {
    Essential,
    Full,
}

fn template(texts: Texts) -> StyledStr {
    let usage = STYLES.get_usage();
    StyledStr::from(format!(
        "{{about-with-newline}}\n{{before-help}}{usage}{}{usage:#} {{usage}}\n\n{{all-args}}{{after-help}}",
        texts.text(Fixed::UsageHeading)
    ))
}

pub(crate) fn print_common_uses(texts: Texts) -> io::Result<()> {
    let uses = format!("{}\n\n{}", texts.common_uses(PROGRAM), texts.all_options(PROGRAM));
    command(Level::Essential, texts)
        .before_help(StyledStr::from(uses))
        .help_template("{about-with-newline}\n{before-help}")
        .print_help()
}

fn command(level: Level, texts: Texts) -> Command {
    let after = match level {
        Level::Essential => texts.all_options(PROGRAM),
        Level::Full => texts.files_produced(),
    };
    Command::new(PROGRAM)
        .version(messages::long_version(
            &media::library::version(),
            media::library::license(),
        ))
        .about(texts.text(Fixed::About))
        .help_template(template(texts))
        .before_help(StyledStr::from(texts.common_uses(PROGRAM)))
        .after_help(StyledStr::from(after))
        .styles(STYLES)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .next_help_heading(texts.text(Fixed::OptionsHeading))
        .args(essential_arguments(texts))
        .args(useful_arguments(level, texts))
        .arg(
            Arg::new("help")
                .short('h')
                .long(HELP)
                .action(ArgAction::HelpShort)
                .help(texts.text(Fixed::HelpHelp)),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::Version)
                .help(texts.text(Fixed::VersionHelp)),
        )
        .next_help_heading(texts.text(Fixed::DetectionHeading))
        .args(advanced_arguments(level, texts))
}

pub(crate) fn parse(
    arguments: impl IntoIterator<Item = impl Into<OsString>>,
    texts: Texts,
) -> Result<Request, clap::Error> {
    let arguments: Vec<OsString> = arguments.into_iter().map(Into::into).collect();
    let mut matches = command(level(&arguments), texts).try_get_matches_from(arguments)?;
    let inputs = many(&mut matches, "inputs");
    let out_dir = matches.remove_one::<PathBuf>("out-dir");
    let options = options(&mut matches, texts)?;
    Ok(Request {
        inputs,
        out_dir,
        options,
        skip_refused: matches.get_flag(SKIP_REFUSED),
    })
}

fn options(matches: &mut ArgMatches, texts: Texts) -> Result<Options, clap::Error> {
    let defaults = DetectSettings::default();
    let settings = DetectSettings {
        threshold: one(matches, Setting::Threshold.flag(), texts)?,
        work_width: one(matches, Setting::WorkWidth.flag(), texts)?,
        osd_regions: many(matches, OSD_REGION),
        ..defaults
    };
    let detect =
        DetectConfig::try_from(&settings).map_err(|error| refusal(texts, &messages::detection_refused(&error)))?;
    let default_track = TrackConfig::default();
    let track = TrackConfig::new(
        default_track.max_jump(),
        default_track.max_gap(),
        one(matches, MIN_HITS, texts)?,
        default_track.min_travel(),
        default_track.twin_distance(),
        default_track.max_median_turn(),
    )
    .map_err(|error| refusal(texts, &messages::tracking_refused(MIN_HITS, &error)))?;
    let stability = StabilityConfig::new(
        NonZeroU32::new(one(matches, MAX_BLOBS, texts)?),
        StabilityConfig::default().pad_s(),
    )
    .map_err(|error| refusal(texts, &messages::tracking_refused(MAX_BLOBS, &error)))?;
    let default_render = RenderConfig::default();
    let choice = one::<Zoom>(matches, "zoom", texts)?;
    let render = RenderConfig::new(
        default_render.box_pad(),
        default_render.trail_s(),
        one(matches, "clip-margin", texts)?,
        one(matches, "max-tracks", texts)?,
    )
    .map_err(|error| refusal(texts, &messages::render_refused(&error)))?
    .with_zoom(ZoomConfig::choosing(choice));
    Ok(Options {
        detect,
        track,
        stability,
        render,
        annotated: matches.get_flag("annotated"),
    })
}

fn one<T: Clone + Send + Sync + 'static>(matches: &mut ArgMatches, id: &str, texts: Texts) -> Result<T, clap::Error> {
    matches
        .remove_one::<T>(id)
        .ok_or_else(|| refusal(texts, &messages::option_refused(id)))
}

fn many<T: Clone + Send + Sync + 'static>(matches: &mut ArgMatches, id: &str) -> Vec<T> {
    matches.remove_many::<T>(id).map(Iterator::collect).unwrap_or_default()
}

pub(crate) fn rejection(texts: Texts, heading: &Message, lines: &[String]) -> clap::Error {
    errors::rejection(texts, heading, lines, &mut command(Level::Essential, texts))
}

pub(crate) fn refusal(texts: Texts, message: &Message) -> clap::Error {
    errors::refusal(texts, message, &mut command(Level::Essential, texts))
}
