use std::path::PathBuf;

use batdetect::access::Access;
use batdetect::failure::Cause;
use batdetect::jobs::{Reason, Refused};
use files::refusals::RefusedInput;
use page::labels::{Label, Labels};
use page::model::Date;
use page::script::Script;
use types::language::Language;

use crate::messages::refused_lines;
use crate::texts::Texts;

const LANGUAGES: [Language; 2] = [Language::French, Language::English];
const DATE: Date = Date {
    year: 2026,
    month: 10,
    day: 16,
    hour: 10,
    minute: 5,
};

pub(crate) fn samples() -> Vec<Label<'static>> {
    let mut samples = vec![
        Label::Language,
        Label::Heading,
        Label::Folder("nuit-12-oct"),
        Label::Videos(3),
        Label::Done(2),
        Label::TooManyCount(1),
        Label::FailedCount(1),
        Label::UnfinishedCount(1),
        Label::WaitingCount(2),
        Label::Tracks(18),
        Label::Reminder,
        Label::ExpandAll,
        Label::CollapseAll,
        Label::Computed(DATE),
        Label::Tried(DATE),
        Label::Length(300),
        Label::NotAnalysed(33),
        Label::FailedBadge,
        Label::TooManyBadge,
        Label::UnfinishedBadge,
        Label::WaitingBadge,
        Label::Failure {
            video: "video_093",
            cause: "not-a-video",
            place: "/nuit",
        },
        Label::Todo {
            cause: "disk-full",
            folder: "/nuit/batdetect",
        },
        Label::Detail("cannot decode"),
        Label::PreviousResults,
        Label::UnfinishedText,
        Label::OlderBadge,
        Label::OlderText,
        Label::WaitingText,
        Label::TooMany {
            video: "video_097",
            count: 301,
            limit: 300,
        },
        Label::TooManyTodo,
        Label::SummaryAlt {
            video: "video_092",
            tracks: 14,
        },
        Label::PartAlt {
            video: "video_097",
            part: 3,
            parts: 8,
            first: 39,
            last: 57,
        },
        Label::Part,
        Label::Parts("video_097"),
        Label::PartTab {
            part: 3,
            parts: 8,
            first: 39,
            last: 57,
        },
        Label::ColumnTrack,
        Label::ColumnStart,
        Label::ColumnLength,
        Label::ColumnVerdict,
        Label::TableCaption("video_092"),
        Label::Player("video_092"),
        Label::TableLink,
        Label::AnnotatedLink,
        Label::Start(4.8),
        Label::Seconds(9.62),
    ];
    samples.extend(refusal_samples());
    samples.extend(Script::ALL.iter().map(|&script| Label::Script(script)));
    samples.push(Label::Day(DATE));
    samples
}

fn refusal_samples() -> Vec<Label<'static>> {
    vec![
        Label::RefusedCount(2),
        Label::RefusedHeading(2),
        Label::RefusedWhen(DATE),
        Label::RefusedConsequence(2),
        Label::RefusedInput {
            path: "/nuit/video_093.mp4",
            reason: "missing",
            cause: None,
            videos: "",
        },
    ]
}

#[test]
fn every_label_of_the_page_reads_in_both_languages_with_all_its_arguments() {
    for language in LANGUAGES {
        let texts = Texts::new(language);
        for label in samples() {
            let holes = match label {
                Label::Script(script) => script.placeholders(),
                _ => &[],
            };
            let said = holes.iter().fold(texts.label(&label), |said, name| {
                said.replace(&format!("{{{name}}}"), "")
            });
            assert!(!said.is_empty(), "{language:?} {label:?}");
            assert!(!said.contains('{'), "{language:?} {label:?}: {said}");
            assert!(!said.starts_with("page-"), "{language:?} {label:?}: {said}");
        }
    }
}

#[test]
fn the_start_of_a_track_reads_to_the_hundredth_with_the_decimal_mark_of_the_language() {
    let (french, english) = (Texts::new(Language::French), Texts::new(Language::English));
    assert_eq!(french.label(&Label::Start(4.8)), "0:04,80");
    assert_eq!(english.label(&Label::Start(209.594)), "3:29.59");
    assert_eq!(french.label(&Label::Start(59.999)), "1:00,00");
}

#[test]
fn a_length_keeps_its_number_and_unit_on_one_line() {
    let french = Texts::new(Language::French);
    assert_eq!(french.label(&Label::Seconds(9.62)), "9,6\u{a0}s");
    assert_eq!(french.label(&Label::Length(300)), "5\u{a0}min\u{a0}0\u{a0}s");
}

#[test]
fn a_date_names_its_month_in_the_language_of_the_page() {
    assert_eq!(
        Texts::new(Language::French).label(&Label::Computed(DATE)),
        "calculée le 16 octobre 2026, 10:05"
    );
    assert_eq!(
        Texts::new(Language::English).label(&Label::Tried(DATE)),
        "tried on 16 October 2026, 10:05"
    );
}

#[test]
fn a_failure_reads_as_the_reason_and_advice_of_the_summary_without_its_indent() {
    let french = Texts::new(Language::French);
    let reason = Label::Failure {
        video: "video_093",
        cause: "not-a-video",
        place: "",
    };
    assert_eq!(
        french.label(&reason),
        "video_093\u{a0}: ce fichier n'est pas une vidéo lisible, ou sa copie est incomplète"
    );
    let todo = french.label(&Label::Todo {
        cause: "not-a-video",
        folder: "",
    });
    assert!(todo.starts_with("à faire\u{a0}: s'il vient des jumelles"), "{todo}");
}

fn every_refusal() -> Vec<Refused> {
    let accesses = [
        Access::Refused,
        Access::NotAFolder,
        Access::Missing,
        Access::ReadOnly,
        Access::DiskFull,
        Access::Unavailable,
    ];
    let causes = [
        Cause::Missing,
        Cause::ReadRefused,
        Cause::NotAVideo,
        Cause::UnusableName,
        Cause::Damaged,
        Cause::Unsupported,
        Cause::Unexpected,
    ];
    let mut reasons = vec![
        Reason::Missing,
        Reason::NotAVideo,
        Reason::NoVideo,
        Reason::SharedFolder {
            videos: vec![PathBuf::from("/nuit/v.mp4"), PathBuf::from("/nuit/V.mp4")],
        },
    ];
    reasons.extend(accesses.map(Reason::Unreadable));
    reasons.extend(causes.map(|cause| Reason::Unopened {
        cause,
        detail: "moov atom not found".to_owned(),
    }));
    reasons
        .into_iter()
        .map(|reason| Refused {
            path: PathBuf::from("/nuit/video_093.mp4"),
            reason,
        })
        .collect()
}

fn page_lines(texts: Texts, input: &RefusedInput) -> Vec<String> {
    let videos = input.videos.join(", ");
    let mut lines = vec![
        texts.label(&Label::RefusedInput {
            path: &input.path,
            reason: &input.reason,
            cause: input.cause.as_deref(),
            videos: &videos,
        }),
        texts.label(&Label::Todo {
            cause: &input.todo,
            folder: &input.path,
        }),
    ];
    lines.extend(
        input
            .detail
            .as_deref()
            .map(|detail| texts.label(&Label::Detail(detail))),
    );
    lines
}

#[test]
fn a_refused_input_reads_on_the_page_as_in_the_terminal_in_both_languages() {
    for language in LANGUAGES {
        let texts = Texts::new(language);
        for refusal in every_refusal() {
            let terminal = refused_lines(texts, std::slice::from_ref(&refusal));
            let input = batdetect::refusals::input(&refusal);
            let terminal: Vec<&str> = terminal.iter().map(|line| line.trim_start()).collect();
            assert_eq!(page_lines(texts, &input), terminal, "{language:?} {refusal:?}");
        }
    }
}

#[test]
fn every_text_of_the_page_script_keeps_the_holes_the_script_fills_in_both_languages() {
    for language in LANGUAGES {
        let texts = Texts::new(language);
        for &script in Script::ALL {
            let text = texts.label(&Label::Script(script));
            for name in script.placeholders() {
                assert!(text.contains(&format!("{{{name}}}")), "{language:?} {script:?} {text}");
            }
        }
    }
}
