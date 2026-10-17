use std::fmt::Debug;

use crate::model::Date;
use crate::script::Script;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Label<'a> {
    Language,
    Heading,
    Folder(&'a str),
    Videos(usize),
    Done(usize),
    TooManyCount(usize),
    FailedCount(usize),
    UnfinishedCount(usize),
    OlderCount(usize),
    WaitingCount(usize),
    NotStartedCount(usize),
    Tracks(usize),
    Reminder,
    ExpandAll,
    CollapseAll,
    Computed(Date),
    Tried(Date),
    Length(u64),
    NotAnalysed(u64),
    FailedBadge,
    TooManyBadge,
    UnfinishedBadge,
    OlderBadge,
    WaitingBadge,
    NotStartedBadge,
    Failure {
        video: &'a str,
        cause: &'a str,
        place: &'a str,
    },
    Todo {
        cause: &'a str,
        folder: &'a str,
    },
    Detail(&'a str),
    RefusedCount(usize),
    RefusedHeading(usize),
    RefusedWhen(Date),
    RefusedConsequence(usize),
    RefusedInput {
        path: &'a str,
        reason: &'a str,
        cause: Option<&'a str>,
        videos: &'a str,
    },
    PreviousResults,
    UnfinishedText,
    OlderText,
    WaitingText,
    NotStartedText,
    TooMany {
        video: &'a str,
        count: usize,
        limit: usize,
    },
    TooManyTodo,
    SummaryAlt {
        video: &'a str,
        tracks: usize,
    },
    PartAlt {
        video: &'a str,
        part: usize,
        parts: usize,
        first: usize,
        last: usize,
    },
    Part,
    Parts(&'a str),
    PartTab {
        part: usize,
        parts: usize,
        first: usize,
        last: usize,
    },
    ColumnTrack,
    ColumnStart,
    ColumnLength,
    ColumnVerdict,
    TableCaption(&'a str),
    Player(&'a str),
    TableLink,
    AnnotatedLink,
    Start(f64),
    Aloud(f64),
    Seconds(f64),
    Scene(SceneText<'a>),
    Script(Script),
    Day(Date),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneText<'a> {
    Alt {
        video: &'a str,
        tracks: usize,
    },
    PartAlt {
        video: &'a str,
        part: usize,
        parts: usize,
        first: usize,
        last: usize,
    },
    Clock(f64),
    Window {
        start: &'a str,
        end: &'a str,
    },
    Span {
        start: &'a str,
        end: &'a str,
    },
    SpanAloud {
        start: &'a str,
        end: &'a str,
    },
    NothingNotAnalysed,
    NothingUnreadable,
    NotAnalysed(&'a str),
    Unreadable(&'a str),
    More(usize),
    Complete,
    NewTab,
    CompleteNewTab,
    BandNotAnalysed,
    BandUnreadable,
    StartOf {
        track: usize,
        start: f64,
    },
}

pub trait Labels: Debug {
    fn label(&self, label: &Label<'_>) -> String;
}

pub(crate) fn filled(labels: &dyn Labels, script: Script, values: &[(&str, &str)]) -> String {
    values
        .iter()
        .fold(labels.label(&Label::Script(script)), |text, (name, value)| {
            text.replace(&format!("{{{name}}}"), value)
        })
}
