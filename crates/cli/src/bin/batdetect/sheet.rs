use batdetect::failure::TOO_MANY_TRACKS;
use batdetect::jobs::{NO_VIDEO, NOT_A_VIDEO, SHARED_FOLDER, UNOPENED};
use batdetect::process::Wording;
use files::params::language_code;
use fluent_bundle::{FluentArgs, FluentValue};
use page::labels::{Label, Labels, SceneText};
use page::model::Date;
use page::script::Script;
use summary::header::{clock, duration};

use crate::messages::extensions;
use crate::numbers::NO_BREAK_SPACE;
use crate::texts::Texts;

#[cfg(test)]
pub(crate) mod tests;

pub(crate) const DATE: &str = "page-date";
const HUNDREDTHS_PER_SECOND: f64 = 100.0;
const HUNDREDTHS_PER_SECOND_WHOLE: u64 = 100;
const HUNDREDTHS_PER_MINUTE: u64 = 6000;
const YES: &str = "yes";
const NO: &str = "no";

#[derive(Debug)]
pub(crate) enum Spoken<'a> {
    Direct(String),
    Message(&'static str, FluentArgs<'a>),
}

impl Labels for Texts {
    fn label(&self, label: &Label<'_>) -> String {
        match self.spoken(label) {
            Spoken::Direct(text) => text,
            Spoken::Message(id, arguments) => self.said(id, &arguments),
        }
    }
}

impl Texts {
    pub(crate) fn spoken<'a>(self, label: &Label<'a>) -> Spoken<'a> {
        match *label {
            Label::Language => Spoken::Direct(language_code(self.language()).to_owned()),
            Label::Day(date) => Spoken::Direct(self.date(date)),
            Label::Script(script) => scripted(script),
            Label::Length(seconds) => Spoken::Direct(unbroken(&duration(seconds))),
            Label::Start(seconds) => Spoken::Direct(self.clock(seconds)),
            Label::Seconds(seconds) => Spoken::Direct(unbroken(&format!("{} s", self.fixed(seconds, 1)))),
            Label::Todo { cause, folder } => Spoken::Direct(self.todo(cause, folder)),
            Label::TooManyTodo => Spoken::Direct(self.todo(TOO_MANY_TRACKS, "")),
            Label::Detail(detail) => {
                Spoken::Direct(self.trimmed("failure-detail", &arguments([("detail", detail.into())])))
            }
            Label::Computed(date) => message("page-computed", [("date", self.date(date).into())]),
            Label::Tried(date) => message("page-tried", [("date", self.date(date).into())]),
            Label::NotAnalysed(seconds) => {
                message("page-not-analysed", [("duration", unbroken(&duration(seconds)).into())])
            }
            Label::Folder(name) => message("page-folder", [("name", name.into())]),
            Label::Videos(count) => message("page-videos", [("count", count.into())]),
            Label::Done(count) => message("tally-done", [("count", count.into())]),
            Label::TooManyCount(count) => message("tally-too-many-count", [("count", count.into())]),
            Label::FailedCount(count) => message("tally-failed-count", [("count", count.into())]),
            Label::UnfinishedCount(count) => message("page-unfinished-count", [("count", count.into())]),
            Label::WaitingCount(count) => message("page-waiting-count", [("count", count.into())]),
            Label::Tracks(count) => message("summary-tracks", [("count", count.into())]),
            Label::Failure { video, cause, place } => failure(video, cause, place),
            Label::RefusedInput {
                path,
                reason,
                cause,
                videos,
            } => refused_input(path, reason, cause, videos),
            Label::RefusedCount(count) => message("tally-refused-count", [("count", count.into())]),
            Label::RefusedHeading(count) => message("page-refused-heading", [("count", count.into())]),
            Label::RefusedWhen(date) => message("page-refused-when", [("date", self.date(date).into())]),
            Label::RefusedConsequence(count) => message("page-refused-consequence", [("count", count.into())]),
            Label::TooMany { video, count, limit } => too_many(video, count, limit),
            Label::SummaryAlt { video, tracks } => summary_alt(video, tracks),
            Label::PartAlt {
                video,
                part,
                parts,
                first,
                last,
            } => parted("page-part-alt", Some(video), [part, parts, first, last]),
            Label::PartTab {
                part,
                parts,
                first,
                last,
            } => parted("page-part-tab", None, [part, parts, first, last]),
            Label::Parts(video) => message("page-parts", [("video", video.into())]),
            Label::TableCaption(video) => message("page-table-caption", [("video", video.into())]),
            Label::Player(video) => message("page-player", [("video", video.into())]),
            Label::Heading => message("page-heading", []),
            Label::Reminder => message("page-reminder", []),
            Label::ExpandAll => message("page-expand", []),
            Label::CollapseAll => message("page-collapse", []),
            Label::FailedBadge => message("tally-failed-cell", []),
            Label::TooManyBadge => message("page-too-many-badge", []),
            Label::UnfinishedBadge => message("page-unfinished-badge", []),
            Label::OlderBadge => message("page-older-badge", []),
            Label::WaitingBadge => message("page-waiting-badge", []),
            Label::PreviousResults => message("page-previous-results", []),
            Label::UnfinishedText => message("page-unfinished", []),
            Label::OlderText => message("page-older", []),
            Label::WaitingText => message("page-waiting", []),
            Label::Part => message("page-part", []),
            Label::ColumnTrack => message("page-column-track", []),
            Label::ColumnStart => message("page-column-start", []),
            Label::ColumnLength => message("page-column-length", []),
            Label::ColumnVerdict => message("page-column-verdict", []),
            Label::TableLink => message("page-table-link", []),
            Label::AnnotatedLink => message("page-annotated-link", []),
            Label::Scene(scene) => self.scene(scene),
        }
    }

    fn scene(self, scene: SceneText<'_>) -> Spoken<'_> {
        let span = |id, start, end| message(id, [("start", clock(start).into()), ("end", clock(end).into())]);
        match scene {
            SceneText::Alt { video, tracks } => {
                message("page-scene-alt", [("video", video.into()), ("tracks", tracks.into())])
            }
            SceneText::PartAlt {
                video,
                part,
                parts,
                first,
                last,
            } => parted("page-scene-part-alt", Some(video), [part, parts, first, last]),
            SceneText::Clock(seconds) => Spoken::Direct(clock(seconds)),
            SceneText::Window { start, end } => span("page-scene-window", start, end),
            SceneText::Span { start, end } => span("page-scene-span", start, end),
            SceneText::NothingNotAnalysed => message("page-scene-nothing-not-analysed", []),
            SceneText::NothingUnreadable => message("page-scene-nothing-unreadable", []),
            SceneText::NotAnalysed(periods) => message("page-scene-not-analysed", [("periods", periods.into())]),
            SceneText::Unreadable(periods) => message("page-scene-unreadable", [("periods", periods.into())]),
            SceneText::More(count) => message("page-scene-more", [("count", count.into())]),
            SceneText::Complete => message("page-scene-complete", []),
            SceneText::NewTab => message("page-scene-new-tab", []),
            SceneText::CompleteNewTab => message("page-scene-complete-new-tab", []),
            SceneText::BandNotAnalysed => message("band-not-analysed", []),
            SceneText::BandUnreadable => message("band-unreadable", []),
            SceneText::StartOf { track, start } => message(
                "page-strip-start",
                [("track", track.into()), ("start", self.clock(start).into())],
            ),
        }
    }

    fn clock(self, seconds: f64) -> String {
        let hundredths = (seconds.max(0.0) * HUNDREDTHS_PER_SECOND).round() as u64;
        let (minutes, rest) = (hundredths / HUNDREDTHS_PER_MINUTE, hundredths % HUNDREDTHS_PER_MINUTE);
        let rest = format!(
            "{:02}.{:02}",
            rest / HUNDREDTHS_PER_SECOND_WHOLE,
            rest % HUNDREDTHS_PER_SECOND_WHOLE
        );
        format!("{minutes}:{}", self.decimal(&rest))
    }

    fn todo(self, cause: &str, folder: &str) -> String {
        self.trimmed(
            "failure-todo",
            &arguments([("cause", cause.into()), ("folder", folder.into())]),
        )
    }

    fn trimmed(self, id: &'static str, values: &FluentArgs<'_>) -> String {
        self.said(id, values).trim_start().to_owned()
    }

    fn date(self, date: Date) -> String {
        let values = arguments([
            ("day", date.day.to_string().into()),
            ("month", i64::from(date.month).into()),
            ("year", date.year.to_string().into()),
            ("hour", date.hour.to_string().into()),
            ("minute", format!("{:02}", date.minute).into()),
        ]);
        self.said(DATE, &values)
    }
}

fn failure<'a>(video: &'a str, cause: &'a str, place: &'a str) -> Spoken<'a> {
    message(
        "failure",
        [
            ("video", video.into()),
            ("cause", cause.into()),
            ("place", place.into()),
        ],
    )
}

fn refused_input<'a>(path: &'a str, reason: &'a str, cause: Option<&'a str>, videos: &'a str) -> Spoken<'a> {
    match (reason, cause) {
        (NOT_A_VIDEO | NO_VIDEO, _) => {
            let (listed, last) = extensions();
            let folder = if reason == NO_VIDEO { YES } else { NO };
            message(
                "input-without-video",
                [
                    ("path", path.into()),
                    ("folder", folder.into()),
                    ("listed", listed.into()),
                    ("last", last.into()),
                ],
            )
        }
        (SHARED_FOLDER, _) => message("shared-folder", [("path", path.into()), ("videos", videos.into())]),
        (UNOPENED, Some(cause)) => failure(path, cause, path),
        _ => message("input-refused", [("path", path.into()), ("reason", reason.into())]),
    }
}

fn too_many(video: &str, count: usize, limit: usize) -> Spoken<'_> {
    message(
        "tally-too-many",
        [
            ("video", video.into()),
            ("count", count.into()),
            ("limit", limit.into()),
        ],
    )
}

fn summary_alt(video: &str, tracks: usize) -> Spoken<'_> {
    message("page-summary-alt", [("video", video.into()), ("tracks", tracks.into())])
}

fn parted<'a>(id: &'static str, video: Option<&'a str>, [part, parts, first, last]: [usize; 4]) -> Spoken<'a> {
    let mut values = arguments([
        ("part", part.into()),
        ("parts", parts.into()),
        ("first", first.into()),
        ("last", last.into()),
    ]);
    if let Some(video) = video {
        values.set("video", video);
    }
    Spoken::Message(id, values)
}

fn scripted<'a>(script: Script) -> Spoken<'a> {
    let mut values = FluentArgs::new();
    for name in script.placeholders() {
        values.set(*name, format!("{{{name}}}"));
    }
    Spoken::Message(script.message_id(), values)
}

fn message<'a, const N: usize>(id: &'static str, values: [(&'static str, FluentValue<'a>); N]) -> Spoken<'a> {
    Spoken::Message(id, arguments(values))
}

fn arguments<'a, const N: usize>(values: [(&'static str, FluentValue<'a>); N]) -> FluentArgs<'a> {
    let mut arguments = FluentArgs::new();
    for (name, value) in values {
        arguments.set(name, value);
    }
    arguments
}

fn unbroken(text: &str) -> String {
    text.replace(' ', &NO_BREAK_SPACE.to_string())
}
