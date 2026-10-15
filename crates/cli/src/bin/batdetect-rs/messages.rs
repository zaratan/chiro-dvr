use std::error::Error;
use std::path::PathBuf;

use batdetect::access::Access;
use batdetect::failure::{self, Cause};
use batdetect::jobs::{self, Job, Reason, Refused};
use batdetect::options::Options;
use batdetect::outcome::{Failure, Panic, Rank};
use batdetect::progress::Step;
use batdetect::writable::Unwritable;
use engine::settings;
use render::config;

use crate::parsing::{Expected, Unreadable};
use crate::texts::{Fixed, Message, Shown, Texts};

#[cfg(test)]
mod tests;

const CAUSE_SEPARATOR: &str = ": ";
const TOO_MANY_TRACKS: &str = "too-many-tracks";
const INTERNAL: &str = "internal";
const UNSUPPORTED: &str = "unsupported";
const ENCODING: &str = "encoding";
const UNEXPECTED: &str = "unexpected";
const REFERS_TO_DETAIL: [&str; 4] = [UNSUPPORTED, ENCODING, UNEXPECTED, INTERNAL];

pub(crate) fn detection_refused(error: &settings::Error) -> Message {
    match *error {
        settings::Error::Setting { setting, value, bound } => Message::SettingRefused {
            option: format!("--{}", setting.flag()),
            bound,
            value: Shown::Number(value),
        },
        settings::Error::Region { index, region } => Message::RegionRefused {
            index: index + 1,
            region: region.to_string(),
        },
        settings::Error::Areas { .. } => Message::Fixed(Fixed::SettingsRefused),
    }
}

pub(crate) fn render_refused(error: &config::Error) -> Message {
    match *error {
        config::Error::Seconds {
            setting: config::Seconds::ClipMargin,
            value,
        } => Message::ClipMarginRefused {
            value: Shown::Number(value),
        },
        config::Error::Seconds {
            setting: config::Seconds::Trail,
            ..
        }
        | config::Error::Threshold { .. } => Message::Fixed(Fixed::SettingsRefused),
    }
}

pub(crate) fn value_refused(unreadable: &Unreadable) -> Message {
    let value = unreadable.written.clone();
    match unreadable.expected {
        Expected::Setting(bound) => Message::SettingRefused {
            option: unreadable.option.clone(),
            bound,
            value: Shown::Written(value),
        },
        Expected::Seconds => Message::ClipMarginRefused {
            value: Shown::Written(value),
        },
        Expected::Region => Message::RegionShape { value },
    }
}

pub(crate) fn tracking_refused(option: &str, error: &tracking::error::Error) -> Message {
    match error {
        tracking::error::Error::Setting { .. } | tracking::error::Error::Track(_) => option_refused(option),
    }
}

pub(crate) fn option_refused(option: &str) -> Message {
    Message::OptionRefused {
        option: format!("--{option}"),
    }
}

pub(crate) fn inputs_refused(texts: Texts, error: &jobs::Error) -> (Message, Vec<String>) {
    let (listed, last) = extensions();
    match error {
        jobs::Error::NoVideo { folders } => (
            Message::NoVideo {
                count: folders.len(),
                listed,
                last,
            },
            folders.iter().map(|folder| folder.display().to_string()).collect(),
        ),
        jobs::Error::Refused { refused } => (
            Message::Fixed(Fixed::InputsRefused),
            refused
                .iter()
                .map(|refusal| texts.say(&input_refused(refusal, &listed, &last)))
                .collect(),
        ),
    }
}

fn extensions() -> (String, String) {
    match jobs::video_extensions().split_last() {
        Some((last, listed)) => (listed.join(", "), (*last).to_owned()),
        None => (String::new(), String::new()),
    }
}

fn input_refused(refusal: &Refused, listed: &str, last: &str) -> Message {
    let path = refusal.path.display().to_string();
    let reason = match &refusal.reason {
        Reason::Missing => "missing",
        Reason::Unreadable(access) => read_access(*access),
        Reason::NotAVideo | Reason::NoVideo => {
            return Message::InputWithoutVideo {
                path,
                folder: refusal.reason == Reason::NoVideo,
                listed: listed.to_owned(),
                last: last.to_owned(),
            };
        }
        Reason::SharedFolder { videos } => {
            let named: Vec<String> = videos.iter().map(|video| video.display().to_string()).collect();
            return Message::SharedFolder {
                path,
                videos: named.join(", "),
            };
        }
    };
    Message::InputRefused { path, reason }
}

const fn read_access(access: Access) -> &'static str {
    match access {
        Access::Missing => "missing",
        Access::Refused => "refused",
        Access::NotAFolder => "not-a-folder",
        Access::ReadOnly | Access::DiskFull | Access::Unavailable => "unavailable",
    }
}

pub(crate) fn folders_refused(texts: Texts, refused: &[Unwritable]) -> (Message, Vec<String>) {
    let lines = refused
        .iter()
        .map(|refusal| {
            texts.say(&Message::FolderRefused {
                path: refusal.folder.display().to_string(),
                access: write_access(refusal.access),
            })
        })
        .collect();
    (Message::FoldersRefused { count: refused.len() }, lines)
}

const fn write_access(access: Access) -> &'static str {
    match access {
        Access::Refused => "refused",
        Access::NotAFolder => "not-a-folder",
        Access::Missing => "missing",
        Access::ReadOnly => "read-only",
        Access::DiskFull => "disk-full",
        Access::Unavailable => "unavailable",
    }
}

pub(crate) fn version() -> &'static str {
    env!("BATDETECT_VERSION")
}

pub(crate) fn long_version(ffmpeg_version: &str, ffmpeg_license: &str) -> String {
    format!("{}\nFFmpeg {ffmpeg_version} — {ffmpeg_license}", version())
}

pub(crate) fn video_step(rank: Rank, job: &Job, step: Step) -> Message {
    Message::VideoStep {
        rank,
        video: short_name(job),
        step,
    }
}

pub(crate) fn video_done(rank: Rank, job: &Job, tracks: usize) -> Message {
    Message::VideoDone {
        rank,
        video: short_name(job),
        tracks,
    }
}

pub(crate) fn video_failed(rank: Rank, job: &Job) -> Message {
    Message::VideoFailed {
        rank,
        video: short_name(job),
    }
}

fn short_name(job: &Job) -> String {
    job.video.file_stem().unwrap_or_default().to_string_lossy().into_owned()
}

pub(crate) fn redone(job: &Job, clips: &[PathBuf], cause: &dyn Error) -> Message {
    let names: Vec<String> = clips
        .iter()
        .map(|clip| clip.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .collect();
    Message::ClipsRedone {
        video: job.video.display().to_string(),
        names: names.join(", "),
        count: clips.len(),
        causes: causes(cause),
    }
}

pub(crate) fn failure_lines(job: &Job, failure: &Failure, options: &Options) -> Vec<Message> {
    let mut lines = failure_reason(job.video.display().to_string(), job, failure, options);
    lines.extend(failure_detail(failure, options).map(|detail| Message::FailureDetail { detail }));
    lines
}

pub(crate) fn failure_reason(video: String, job: &Job, failure: &Failure, options: &Options) -> Vec<Message> {
    let (cause, place) = match failure {
        Failure::Error(error) => match failure::cause(error, options) {
            Cause::TooManyTracks {
                count,
                limit,
                annotated,
            } => {
                return vec![
                    Message::TooManyTracks {
                        video,
                        count,
                        limit,
                        annotated,
                    },
                    todo(TOO_MANY_TRACKS, job),
                ];
            }
            cause => (named(&cause), place_of(cause, job)),
        },
        Failure::Panic(_) => (INTERNAL, job.dest.clone()),
    };
    vec![
        Message::Failure {
            video,
            cause,
            place: place.display().to_string(),
        },
        todo(cause, job),
    ]
}

fn failure_detail(failure: &Failure, options: &Options) -> Option<String> {
    match failure {
        Failure::Error(error) => match failure::cause(error, options) {
            Cause::TooManyTracks { .. } => None,
            _ => Some(causes(error)),
        },
        Failure::Panic(panic) => Some(panicked(panic)),
    }
}

pub(crate) fn without_detail(message: Message) -> Message {
    match message {
        Message::FailureTodo { cause, .. } if REFERS_TO_DETAIL.contains(&cause) => Message::TallyFailureTodo { cause },
        other => other,
    }
}

pub(crate) fn too_many_todo(job: &Job) -> Message {
    todo(TOO_MANY_TRACKS, job)
}

fn place_of(cause: Cause, job: &Job) -> PathBuf {
    match cause {
        Cause::InUse { file } => file,
        _ => job.dest.clone(),
    }
}

fn todo(cause: &'static str, job: &Job) -> Message {
    Message::FailureTodo {
        cause,
        folder: job.dest.display().to_string(),
    }
}

fn panicked(panic: &Panic) -> String {
    match &panic.location {
        Some(location) => format!("{} ({location})", panic.message),
        None => panic.message.clone(),
    }
}

fn named(cause: &Cause) -> &'static str {
    match cause {
        Cause::Missing => "missing",
        Cause::ReadRefused => "read-refused",
        Cause::NotAVideo => "not-a-video",
        Cause::UnusableName => "unusable-name",
        Cause::Damaged => "damaged",
        Cause::Unsupported => UNSUPPORTED,
        Cause::Settings => "settings",
        Cause::DiskFull => "disk-full",
        Cause::Forbidden => "forbidden",
        Cause::InUse { .. } => "in-use",
        Cause::Encoding => ENCODING,
        Cause::TooManyTracks { .. } => TOO_MANY_TRACKS,
        Cause::Unexpected => UNEXPECTED,
    }
}

pub(crate) fn causes(error: &dyn Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        text.push_str(CAUSE_SEPARATOR);
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}
