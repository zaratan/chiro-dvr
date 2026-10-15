use std::fmt;
use std::sync::OnceLock;

use batdetect::outcome::Rank;
use batdetect::process::Wording;
use batdetect::progress::Step;
use engine::settings::Bound;
use files::names::Part;
use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource, FluentValue};
use summary::header::{Band, HeaderLine, Labels, duration};
use types::language::Language;
use unic_langid::LanguageIdentifier;

use crate::numbers::{self, NARROW_NO_BREAK_SPACE, NO_BREAK_SPACE};

#[cfg(test)]
mod tests;

const FRENCH_SOURCE: &str = include_str!("../../../locales/fr.ftl");
const ENGLISH_SOURCE: &str = include_str!("../../../locales/en.ftl");
const YES: &str = "yes";
const NO: &str = "no";

static FRENCH: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static ENGLISH: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fixed {
    BandNotAnalysed,
    BandUnreadable,
    About,
    InputsHelp,
    OutDirHelp,
    ZoomHelp,
    AnnotatedHelp,
    MaxTracksHelp,
    ClipMarginHelp,
    OsdRegionHelp,
    HelpHelp,
    VersionHelp,
    NoRegion,
    BesideTheVideos,
    ThresholdHelp,
    MinHitsHelp,
    MaxBlobsHelp,
    WorkWidthHelp,
    ValueVideo,
    ValueFolder,
    ValueWhich,
    ValueCount,
    ValueSeconds,
    ValueLevel,
    ValuePixels,
    UsageHeading,
    ArgumentsHeading,
    OptionsHeading,
    DetectionHeading,
    ErrorLabel,
    TipLabel,
    MissingArguments,
    InvalidUtf8,
    NoVideo,
    TallyVideo,
    TallyLength,
    TallyTracks,
    TallyNotAnalysed,
    TallyCompute,
    TallyFailedCell,
    TallyTooManyHeading,
}

impl Fixed {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::BandNotAnalysed => "band-not-analysed",
            Self::BandUnreadable => "band-unreadable",
            Self::About => "about",
            Self::InputsHelp => "inputs-help",
            Self::OutDirHelp => "out-dir-help",
            Self::ZoomHelp => "zoom-help",
            Self::AnnotatedHelp => "annotated-help",
            Self::MaxTracksHelp => "max-tracks-help",
            Self::ClipMarginHelp => "clip-margin-help",
            Self::OsdRegionHelp => "osd-region-help",
            Self::HelpHelp => "help-help",
            Self::VersionHelp => "version-help",
            Self::NoRegion => "no-region",
            Self::BesideTheVideos => "beside-the-videos",
            Self::ThresholdHelp => "threshold-help",
            Self::MinHitsHelp => "min-hits-help",
            Self::MaxBlobsHelp => "max-blobs-help",
            Self::WorkWidthHelp => "work-width-help",
            Self::ValueVideo => "value-video",
            Self::ValueFolder => "value-folder",
            Self::ValueWhich => "value-which",
            Self::ValueCount => "value-count",
            Self::ValueSeconds => "value-seconds",
            Self::ValueLevel => "value-level",
            Self::ValuePixels => "value-pixels",
            Self::UsageHeading => "usage-heading",
            Self::ArgumentsHeading => "arguments-heading",
            Self::OptionsHeading => "options-heading",
            Self::DetectionHeading => "detection-heading",
            Self::ErrorLabel => "error-label",
            Self::TipLabel => "tip-label",
            Self::MissingArguments => "missing-arguments",
            Self::InvalidUtf8 => "invalid-utf8",
            Self::NoVideo => "no-video",
            Self::TallyVideo => "tally-video",
            Self::TallyLength => "tally-length",
            Self::TallyTracks => "tally-tracks",
            Self::TallyNotAnalysed => "tally-not-analysed",
            Self::TallyCompute => "tally-compute",
            Self::TallyFailedCell => "tally-failed-cell",
            Self::TallyTooManyHeading => "tally-too-many-heading",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Message {
    Fixed(Fixed),
    TooManyTracks {
        video: String,
        count: usize,
        limit: usize,
        annotated: bool,
    },
    ClipsRedone {
        video: String,
        names: String,
        count: usize,
        causes: String,
    },
    VideoStep {
        rank: Rank,
        video: String,
        step: Step,
    },
    VideoDone {
        rank: Rank,
        video: String,
        tracks: usize,
    },
    VideoFailed {
        rank: Rank,
        video: String,
    },
    Failure {
        video: String,
        cause: &'static str,
        place: String,
    },
    FailureTodo {
        cause: &'static str,
        folder: String,
    },
    FailureDetail {
        detail: String,
    },
    TallyTitle {
        total: usize,
    },
    TallyDone {
        count: usize,
    },
    TallyTooManyCount {
        count: usize,
    },
    TallyFailedCount {
        count: usize,
    },
    TallyTooManyCell {
        count: usize,
    },
    TallyTooMany {
        video: String,
        count: usize,
        limit: usize,
    },
    TallyFailedHeading {
        failed: usize,
        total: usize,
    },
    TallyKeptHeading {
        count: usize,
    },
    TallyKept {
        video: String,
        table: String,
        written: bool,
    },
    TallyChanged {
        settings: String,
    },
    TallyRegions {
        count: usize,
    },
    TallyFailureTodo {
        cause: &'static str,
    },
    SummaryPart {
        number: usize,
        count: usize,
    },
    SummaryTracks {
        count: usize,
    },
    SummaryNotAnalysed {
        start: String,
        end: String,
    },
    SummaryMore {
        count: usize,
    },
    SummaryUnreadable {
        start: String,
        end: String,
    },
    SummaryMoreUnreadable {
        count: usize,
        duration: String,
        seconds: u64,
    },
    DefaultValue {
        value: String,
    },
    PossibleValues {
        values: String,
    },
    UnexpectedArgument {
        argument: String,
    },
    SimilarArgument {
        suggestion: String,
    },
    SimilarValue {
        suggestion: String,
    },
    ArgumentAsValue {
        argument: String,
        escaped: String,
    },
    InvalidValue {
        value: String,
        argument: String,
    },
    InvalidValueBecause {
        value: String,
        argument: String,
        cause: String,
    },
    ValueRequired {
        argument: String,
    },
    UsedTwice {
        argument: String,
    },
    UnexpectedValue {
        value: String,
        argument: String,
    },
    ArgumentsRefused {
        cause: String,
    },
    MoreInformation {
        flag: String,
    },
    SettingRefused {
        option: String,
        bound: Bound,
        value: f64,
    },
    RegionRefused {
        index: usize,
        region: String,
    },
    ClipMarginRefused {
        value: f64,
    },
    SettingsRefused {
        causes: String,
    },
    VideosRefused {
        causes: String,
    },
    FoldersRefused {
        folders: String,
    },
}

impl Message {
    pub(crate) fn id(&self) -> &'static str {
        match self {
            Self::Fixed(fixed) => fixed.id(),
            Self::TooManyTracks { .. } => "too-many-tracks",
            Self::ClipsRedone { .. } => "clips-redone",
            Self::VideoStep { .. } => "video-step",
            Self::VideoDone { .. } => "video-done",
            Self::VideoFailed { .. } => "video-failed",
            Self::Failure { .. } => "failure",
            Self::FailureTodo { .. } => "failure-todo",
            Self::FailureDetail { .. } => "failure-detail",
            Self::TallyTitle { .. } => "tally-title",
            Self::TallyDone { .. } => "tally-done",
            Self::TallyTooManyCount { .. } => "tally-too-many-count",
            Self::TallyFailedCount { .. } => "tally-failed-count",
            Self::TallyTooManyCell { .. } => "tally-too-many-cell",
            Self::TallyTooMany { .. } => "tally-too-many",
            Self::TallyFailedHeading { .. } => "tally-failed-heading",
            Self::TallyKeptHeading { .. } => "tally-kept-heading",
            Self::TallyKept { .. } => "tally-kept",
            Self::TallyChanged { .. } => "tally-changed",
            Self::TallyRegions { .. } => "tally-regions",
            Self::TallyFailureTodo { .. } => "tally-failure-todo",
            Self::SummaryPart { .. } => "summary-part",
            Self::SummaryTracks { .. } => "summary-tracks",
            Self::SummaryNotAnalysed { .. } => "summary-not-analysed",
            Self::SummaryMore { .. } => "summary-more",
            Self::SummaryUnreadable { .. } => "summary-unreadable",
            Self::SummaryMoreUnreadable { .. } => "summary-more-unreadable",
            Self::DefaultValue { .. } => "default-value",
            Self::PossibleValues { .. } => "possible-values",
            Self::UnexpectedArgument { .. } => "unexpected-argument",
            Self::SimilarArgument { .. } => "similar-argument",
            Self::SimilarValue { .. } => "similar-value",
            Self::ArgumentAsValue { .. } => "argument-as-value",
            Self::InvalidValue { .. } => "invalid-value",
            Self::InvalidValueBecause { .. } => "invalid-value-because",
            Self::ValueRequired { .. } => "value-required",
            Self::UsedTwice { .. } => "used-twice",
            Self::UnexpectedValue { .. } => "unexpected-value",
            Self::ArgumentsRefused { .. } => "arguments-refused",
            Self::MoreInformation { .. } => "more-information",
            Self::SettingRefused { .. } => "setting-refused",
            Self::RegionRefused { .. } => "region-refused",
            Self::ClipMarginRefused { .. } => "clip-margin-refused",
            Self::SettingsRefused { .. } => "settings-refused",
            Self::VideosRefused { .. } => "videos-refused",
            Self::FoldersRefused { .. } => "folders-refused",
        }
    }

    pub(crate) fn arguments(&self) -> FluentArgs<'_> {
        let mut arguments = FluentArgs::new();
        match self {
            Self::TooManyTracks {
                video,
                count,
                limit,
                annotated,
            } => {
                arguments.set("video", video.as_str());
                arguments.set("count", *count);
                arguments.set("limit", *limit);
                arguments.set("annotated", if *annotated { YES } else { NO });
            }
            Self::ClipsRedone {
                video,
                names,
                count,
                causes,
            } => {
                arguments.set("video", video.as_str());
                arguments.set("names", names.as_str());
                arguments.set("count", *count);
                arguments.set("causes", causes.as_str());
            }
            Self::VideoStep { rank, video, step } => {
                ranked(&mut arguments, *rank, video);
                arguments.set("step", step_name(*step));
            }
            Self::VideoDone { rank, video, tracks } => {
                ranked(&mut arguments, *rank, video);
                arguments.set("tracks", *tracks);
            }
            Self::VideoFailed { rank, video } => ranked(&mut arguments, *rank, video),
            Self::Failure { video, cause, place } => {
                arguments.set("video", video.as_str());
                arguments.set("cause", *cause);
                arguments.set("place", place.as_str());
            }
            Self::FailureTodo { cause, folder } => {
                arguments.set("cause", *cause);
                arguments.set("folder", folder.as_str());
            }
            Self::FailureDetail { detail } => arguments.set("detail", detail.as_str()),
            Self::SummaryPart { number, count } => {
                arguments.set("number", *number);
                arguments.set("count", *count);
            }
            Self::SummaryTracks { count } | Self::SummaryMore { count } => arguments.set("count", *count),
            Self::SummaryNotAnalysed { start, end } | Self::SummaryUnreadable { start, end } => {
                arguments.set("start", start.as_str());
                arguments.set("end", end.as_str());
            }
            Self::SummaryMoreUnreadable {
                count,
                duration,
                seconds,
            } => {
                arguments.set("count", *count);
                arguments.set("duration", duration.as_str());
                arguments.set("seconds", *seconds);
            }
            other => other.tally_arguments(&mut arguments),
        }
        arguments
    }

    fn tally_arguments<'a>(&'a self, arguments: &mut FluentArgs<'a>) {
        match self {
            Self::TallyTitle { total } => arguments.set("total", *total),
            Self::TallyDone { count }
            | Self::TallyTooManyCount { count }
            | Self::TallyFailedCount { count }
            | Self::TallyTooManyCell { count }
            | Self::TallyKeptHeading { count }
            | Self::TallyRegions { count } => arguments.set("count", *count),
            Self::TallyTooMany { video, count, limit } => {
                arguments.set("video", video.as_str());
                arguments.set("count", *count);
                arguments.set("limit", *limit);
            }
            Self::TallyFailedHeading { failed, total } => {
                arguments.set("failed", *failed);
                arguments.set("total", *total);
            }
            Self::TallyKept { video, table, written } => {
                arguments.set("video", video.as_str());
                arguments.set("table", table.as_str());
                arguments.set("written", if *written { YES } else { NO });
            }
            Self::TallyChanged { settings } => arguments.set("settings", settings.as_str()),
            Self::TallyFailureTodo { cause } => arguments.set("cause", *cause),
            other => other.command_line_arguments(arguments),
        }
    }

    fn command_line_arguments<'a>(&'a self, arguments: &mut FluentArgs<'a>) {
        match self {
            Self::DefaultValue { value } => arguments.set("value", value.as_str()),
            Self::PossibleValues { values } => arguments.set("values", values.as_str()),
            Self::ClipMarginRefused { value } => arguments.set("value", *value),
            Self::UnexpectedArgument { argument } | Self::ValueRequired { argument } | Self::UsedTwice { argument } => {
                arguments.set("argument", argument.as_str());
            }
            Self::SimilarArgument { suggestion } | Self::SimilarValue { suggestion } => {
                arguments.set("suggestion", suggestion.as_str());
            }
            Self::ArgumentAsValue { argument, escaped } => {
                arguments.set("argument", argument.as_str());
                arguments.set("escaped", escaped.as_str());
            }
            Self::InvalidValue { value, argument } | Self::UnexpectedValue { value, argument } => {
                arguments.set("value", value.as_str());
                arguments.set("argument", argument.as_str());
            }
            Self::InvalidValueBecause { value, argument, cause } => {
                arguments.set("value", value.as_str());
                arguments.set("argument", argument.as_str());
                arguments.set("cause", cause.as_str());
            }
            Self::ArgumentsRefused { cause } => arguments.set("cause", cause.as_str()),
            Self::MoreInformation { flag } => arguments.set("flag", flag.as_str()),
            Self::SettingRefused { option, bound, value } => {
                arguments.set("option", option.as_str());
                let (name, limit) = selected(*bound);
                arguments.set("bound", name);
                arguments.set("limit", limit);
                arguments.set("value", *value);
            }
            Self::RegionRefused { index, region } => {
                arguments.set("index", *index);
                arguments.set("region", region.as_str());
            }
            Self::SettingsRefused { causes } | Self::VideosRefused { causes } => {
                arguments.set("causes", causes.as_str());
            }
            Self::FoldersRefused { folders } => arguments.set("folders", folders.as_str()),
            Self::Fixed(_)
            | Self::TooManyTracks { .. }
            | Self::ClipsRedone { .. }
            | Self::VideoStep { .. }
            | Self::VideoDone { .. }
            | Self::VideoFailed { .. }
            | Self::Failure { .. }
            | Self::FailureTodo { .. }
            | Self::FailureDetail { .. }
            | Self::TallyTitle { .. }
            | Self::TallyDone { .. }
            | Self::TallyTooManyCount { .. }
            | Self::TallyFailedCount { .. }
            | Self::TallyTooManyCell { .. }
            | Self::TallyTooMany { .. }
            | Self::TallyFailedHeading { .. }
            | Self::TallyKeptHeading { .. }
            | Self::TallyKept { .. }
            | Self::TallyChanged { .. }
            | Self::TallyRegions { .. }
            | Self::TallyFailureTodo { .. }
            | Self::SummaryPart { .. }
            | Self::SummaryTracks { .. }
            | Self::SummaryNotAnalysed { .. }
            | Self::SummaryMore { .. }
            | Self::SummaryUnreadable { .. }
            | Self::SummaryMoreUnreadable { .. } => {}
        }
    }
}

fn ranked<'a>(arguments: &mut FluentArgs<'a>, rank: Rank, video: &'a str) {
    arguments.set("position", rank.position);
    arguments.set("total", rank.total);
    arguments.set("video", video);
}

fn step_name(step: Step) -> &'static str {
    match step {
        Step::Detection => "detection",
        Step::Tracking => "tracking",
        Step::Summary => "summary",
        Step::Clips => "clips",
    }
}

fn selected(bound: Bound) -> (&'static str, u32) {
    match bound {
        Bound::Positive => ("positive", 0),
        Bound::NotNegative => ("not-negative", 0),
        Bound::AtLeast(limit) => ("at-least", limit),
        Bound::UpTo(limit) => ("up-to", limit),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Texts {
    language: Language,
}

impl Texts {
    pub(crate) const fn new(language: Language) -> Self {
        Self { language }
    }

    pub(crate) fn number(self, value: f64) -> String {
        self.grouped(value)
    }

    pub(crate) fn count(self, value: usize) -> String {
        self.grouped(value)
    }

    fn grouped(self, value: impl fmt::Display) -> String {
        match self.language {
            Language::French => numbers::french(&value.to_string()),
            Language::English => numbers::english(&value.to_string()),
        }
    }

    pub(crate) fn text(self, fixed: Fixed) -> String {
        self.say(&Message::Fixed(fixed))
    }

    pub(crate) fn say(self, message: &Message) -> String {
        match resolved(self.language, message) {
            Ok(text) | Err(Unresolved::Format { text, .. }) => text,
            Err(Unresolved::Missing(id)) => id.to_owned(),
        }
    }
}

impl Labels for Texts {
    fn line(&self, line: &HeaderLine) -> String {
        let message = match line {
            HeaderLine::Name(name) => return name.clone(),
            HeaderLine::Window { start, end } => return format!("{start} - {end}"),
            HeaderLine::Part(Part { number, count }) => Message::SummaryPart {
                number: *number,
                count: *count,
            },
            HeaderLine::Tracks(count) => Message::SummaryTracks { count: *count },
            HeaderLine::NotAnalysed { start, end } => Message::SummaryNotAnalysed {
                start: start.clone(),
                end: end.clone(),
            },
            HeaderLine::More(count) => Message::SummaryMore { count: *count },
            HeaderLine::Unreadable { start, end } => Message::SummaryUnreadable {
                start: start.clone(),
                end: end.clone(),
            },
            HeaderLine::MoreUnreadable { count, total_seconds } => Message::SummaryMoreUnreadable {
                count: *count,
                duration: duration(*total_seconds),
                seconds: *total_seconds,
            },
        };
        drawable(&self.say(&message))
    }

    fn band(&self, band: Band) -> String {
        let message = match band {
            Band::NotAnalysed => Fixed::BandNotAnalysed,
            Band::Unreadable => Fixed::BandUnreadable,
        };
        drawable(&self.text(message))
    }
}

impl Wording for Texts {
    fn language(&self) -> Language {
        self.language
    }
}

fn drawable(text: &str) -> String {
    text.replace(NARROW_NO_BREAK_SPACE, &NO_BREAK_SPACE.to_string())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Unresolved {
    #[error("no message {0}")]
    Missing(&'static str),
    #[error("{id} reads {text} with {errors:?}")]
    Format {
        id: &'static str,
        text: String,
        errors: Vec<FluentError>,
    },
}

pub(crate) fn resolved(language: Language, message: &Message) -> Result<String, Unresolved> {
    let bundle = bundle(language);
    let id = message.id();
    let pattern = bundle
        .get_message(id)
        .and_then(|found| found.value())
        .ok_or(Unresolved::Missing(id))?;
    let mut errors = Vec::new();
    let text = bundle
        .format_pattern(pattern, Some(&message.arguments()), &mut errors)
        .into_owned();
    if errors.is_empty() {
        Ok(text)
    } else {
        Err(Unresolved::Format { id, text, errors })
    }
}

fn bundle(language: Language) -> &'static FluentBundle<FluentResource> {
    match language {
        Language::French => FRENCH.get_or_init(|| {
            let mut bundle = built("fr", FRENCH_SOURCE);
            bundle.set_formatter(Some(french_number));
            bundle
        }),
        Language::English => ENGLISH.get_or_init(|| {
            let mut bundle = built("en", ENGLISH_SOURCE);
            bundle.set_formatter(Some(english_number));
            bundle
        }),
    }
}

fn built(language: &str, source: &str) -> FluentBundle<FluentResource> {
    let resource = FluentResource::try_new(source.to_owned()).unwrap_or_else(|(resource, _)| resource);
    let mut bundle = FluentBundle::new_concurrent(vec![language.parse::<LanguageIdentifier>().unwrap_or_default()]);
    bundle.set_use_isolating(false);
    bundle.add_resource_overriding(resource);
    bundle
}

fn french_number<M>(value: &FluentValue<'_>, _: &M) -> Option<String> {
    number_text(value).map(|text| numbers::french(&text))
}

fn english_number<M>(value: &FluentValue<'_>, _: &M) -> Option<String> {
    number_text(value).map(|text| numbers::english(&text))
}

fn number_text(value: &FluentValue<'_>) -> Option<String> {
    match value {
        FluentValue::Number(number) => Some(number.as_string().into_owned()),
        _ => None,
    }
}
