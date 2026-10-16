use batdetect::process::Wording;
use files::params::language_code;
use fluent_bundle::FluentArgs;
use page::labels::{Label, Labels};
use page::model::Date;
use summary::header::duration;

use crate::numbers::NO_BREAK_SPACE;
use crate::texts::Texts;

#[cfg(test)]
pub(crate) mod tests;

pub(crate) const DATE: &str = "page-date";
const TOO_MANY_TRACKS: &str = "too-many-tracks";
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
        if let Some(text) = self.direct(label) {
            return Spoken::Direct(text);
        }
        if let Some(id) = plain(label) {
            return Spoken::Message(id, FluentArgs::new());
        }
        let mut arguments = FluentArgs::new();
        let id = self.with_arguments(label, &mut arguments);
        Spoken::Message(id, arguments)
    }

    fn direct(self, label: &Label<'_>) -> Option<String> {
        let text = match *label {
            Label::Language => language_code(self.language()).to_owned(),
            Label::Length(seconds) => unbroken(&duration(seconds)),
            Label::Start(seconds) => self.clock(seconds),
            Label::Seconds(seconds) => unbroken(&format!("{} s", self.fixed(seconds, 1))),
            Label::Todo { cause, folder } => self.todo(cause, folder),
            Label::TooManyTodo => self.todo(TOO_MANY_TRACKS, ""),
            Label::Detail(detail) => {
                let mut arguments = FluentArgs::new();
                arguments.set("detail", detail);
                self.said("failure-detail", &arguments).trim_start().to_owned()
            }
            _ => return None,
        };
        Some(text)
    }

    fn with_arguments<'a>(self, label: &Label<'a>, arguments: &mut FluentArgs<'a>) -> &'static str {
        match *label {
            Label::Computed(date) => given_text(arguments, "date", self.date(date), "page-computed"),
            Label::Tried(date) => given_text(arguments, "date", self.date(date), "page-tried"),
            Label::Folder(name) => given(arguments, "name", name, "page-folder"),
            Label::Videos(count) => counted(arguments, count, "page-videos"),
            Label::Done(count) => counted(arguments, count, "tally-done"),
            Label::TooManyCount(count) => counted(arguments, count, "tally-too-many-count"),
            Label::FailedCount(count) => counted(arguments, count, "tally-failed-count"),
            Label::UnfinishedCount(count) => counted(arguments, count, "page-unfinished-count"),
            Label::WaitingCount(count) => counted(arguments, count, "page-waiting-count"),
            Label::Tracks(count) => counted(arguments, count, "summary-tracks"),
            Label::NotAnalysed(seconds) => {
                given_text(arguments, "duration", unbroken(&duration(seconds)), "page-not-analysed")
            }
            Label::Failure { video, cause, place } => {
                arguments.set("video", video);
                arguments.set("cause", cause);
                arguments.set("place", place);
                "failure"
            }
            Label::TooMany { video, count, limit } => {
                arguments.set("video", video);
                arguments.set("count", count);
                arguments.set("limit", limit);
                "tally-too-many"
            }
            Label::SummaryAlt { video, tracks } => {
                arguments.set("tracks", tracks);
                given(arguments, "video", video, "page-summary-alt")
            }
            Label::PartAlt {
                video,
                part,
                parts,
                first,
                last,
            } => {
                part_arguments(arguments, part, parts, first, last);
                given(arguments, "video", video, "page-part-alt")
            }
            Label::PartTab {
                part,
                parts,
                first,
                last,
            } => {
                part_arguments(arguments, part, parts, first, last);
                "page-part-tab"
            }
            Label::Parts(video) => given(arguments, "video", video, "page-parts"),
            Label::TableCaption(video) => given(arguments, "video", video, "page-table-caption"),
            Label::Player(video) => given(arguments, "video", video, "page-player"),
            Label::PlayClip(track) => numbered(arguments, track, "page-play-clip"),
            Label::PlayZoom(track) => numbered(arguments, track, "page-play-zoom"),
            Label::ShowTrack(track) => numbered(arguments, track, "page-show-track"),
            Label::Playing {
                track,
                total,
                start,
                zoom,
            } => {
                arguments.set("total", total);
                arguments.set("start", start);
                arguments.set("zoom", if zoom { YES } else { NO });
                numbered(arguments, track, "page-playing")
            }
            _ => "",
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
        let mut arguments = FluentArgs::new();
        arguments.set("cause", cause);
        arguments.set("folder", folder);
        self.said("failure-todo", &arguments).trim_start().to_owned()
    }

    fn date(self, date: Date) -> String {
        let mut arguments = FluentArgs::new();
        arguments.set("day", date.day.to_string());
        arguments.set("month", i64::from(date.month));
        arguments.set("year", date.year.to_string());
        arguments.set("hour", date.hour.to_string());
        arguments.set("minute", format!("{:02}", date.minute));
        self.said(DATE, &arguments)
    }
}

fn plain(label: &Label<'_>) -> Option<&'static str> {
    let id = match *label {
        Label::Heading => "page-heading",
        Label::Reminder => "page-reminder",
        Label::ExpandAll => "page-expand",
        Label::CollapseAll => "page-collapse",
        Label::FailedBadge => "tally-failed-cell",
        Label::UnfinishedBadge => "page-unfinished-badge",
        Label::WaitingBadge => "page-waiting-badge",
        Label::PreviousResults | Label::PreviousOf(_) => "page-previous-results",
        Label::UnfinishedText => "page-unfinished",
        Label::WaitingText => "page-waiting",
        Label::Part => "page-part",
        Label::ColumnTrack => "page-column-track",
        Label::ColumnStart => "page-column-start",
        Label::ColumnLength => "page-column-length",
        Label::ColumnVerdict => "page-column-verdict",
        Label::ColumnWatch => "page-column-watch",
        Label::NoVerdict => "page-no-verdict",
        Label::Clip => "page-clip",
        Label::Zoom => "page-zoom",
        Label::Unreadable => "page-unreadable",
        Label::Back => "page-back",
        Label::Previous => "page-previous",
        Label::Next => "page-next",
        Label::PreviousClip => "page-previous-clip",
        Label::NextClip => "page-next-clip",
        Label::TableLink => "page-table-link",
        Label::AnnotatedLink => "page-annotated-link",
        _ => return None,
    };
    Some(id)
}

fn given_text(arguments: &mut FluentArgs<'_>, name: &'static str, value: String, id: &'static str) -> &'static str {
    arguments.set(name, value);
    id
}

fn counted(arguments: &mut FluentArgs<'_>, count: usize, id: &'static str) -> &'static str {
    arguments.set("count", count);
    id
}

fn given<'a>(arguments: &mut FluentArgs<'a>, name: &'static str, value: &'a str, id: &'static str) -> &'static str {
    arguments.set(name, value);
    id
}

fn numbered(arguments: &mut FluentArgs<'_>, track: usize, id: &'static str) -> &'static str {
    arguments.set("track", track);
    id
}

fn part_arguments(arguments: &mut FluentArgs<'_>, part: usize, parts: usize, first: usize, last: usize) {
    arguments.set("part", part);
    arguments.set("parts", parts);
    arguments.set("first", first);
    arguments.set("last", last);
}

fn unbroken(text: &str) -> String {
    text.replace(' ', &NO_BREAK_SPACE.to_string())
}
