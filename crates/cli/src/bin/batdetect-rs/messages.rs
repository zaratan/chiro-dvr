use std::error::Error;

use batdetect::jobs::Job;
use files::names::Part;
use summary::header::{Band, HeaderLine, Labels, duration};
use types::language::Language;

#[cfg(test)]
mod tests;

pub(crate) const ABOUT: &str = "Detects and tracks bats in thermal binocular videos";
pub(crate) const INPUTS_HELP: &str = "videos, or folders whose videos are all processed";
pub(crate) const OUTPUT_HELP: &str = "folder that receives one folder of results per video";
pub(crate) const NO_VIDEO: &str = "no video found";
pub(crate) const NO_OUTPUT: &str = "the output folder is missing";
const CAUSE_SEPARATOR: &str = ": ";

pub(crate) fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub(crate) fn long_version(ffmpeg_version: &str, ffmpeg_license: &str) -> String {
    format!("{}\nFFmpeg {ffmpeg_version} — {ffmpeg_license}", version())
}

pub(crate) fn processed(language: Language, job: &Job, tracks: usize) -> String {
    let video = job.video.file_name().unwrap_or_default().to_string_lossy();
    let dest = job.dest.display();
    match (language, tracks) {
        (Language::French, 0 | 1) => format!("{video} : {tracks} piste -> {dest}"),
        (Language::French, _) => format!("{video} : {tracks} pistes -> {dest}"),
        (Language::English, 1) => format!("{video}: 1 track -> {dest}"),
        (Language::English, _) => format!("{video}: {tracks} tracks -> {dest}"),
    }
}

pub(crate) fn failed(job: &Job, error: &dyn Error) -> String {
    format!("{}{CAUSE_SEPARATOR}{}", job.video.display(), causes(error))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SummaryLabels(pub(crate) Language);

impl Labels for SummaryLabels {
    fn line(&self, line: &HeaderLine) -> String {
        match (self.0, line) {
            (_, HeaderLine::Name(name)) => name.clone(),
            (_, HeaderLine::Window { start, end }) => format!("{start} - {end}"),
            (Language::French, HeaderLine::Part(Part { number, count })) => format!("partie {number} / {count}"),
            (Language::English, HeaderLine::Part(Part { number, count })) => format!("part {number} / {count}"),
            (Language::French, HeaderLine::Tracks(count @ (0 | 1))) => format!("{count} piste"),
            (Language::French, HeaderLine::Tracks(count)) => format!("{count} pistes"),
            (Language::English, HeaderLine::Tracks(1)) => "1 track".to_owned(),
            (Language::English, HeaderLine::Tracks(count)) => format!("{count} tracks"),
            (Language::French, HeaderLine::NotAnalysed { start, end }) => format!("hors analyse {start} - {end}"),
            (Language::English, HeaderLine::NotAnalysed { start, end }) => format!("not analysed {start} - {end}"),
            (Language::French, HeaderLine::More(count @ (0 | 1))) => format!("+ {count} autre"),
            (Language::French, HeaderLine::More(count)) => format!("+ {count} autres"),
            (Language::English, HeaderLine::More(count)) => format!("+ {count} more"),
            (Language::French, HeaderLine::Unreadable { start, end }) => format!("illisible {start} - {end}"),
            (Language::English, HeaderLine::Unreadable { start, end }) => format!("unreadable {start} - {end}"),
            (Language::French, HeaderLine::MoreUnreadable { count, total_seconds }) => {
                let others = if *count > 1 { "autres" } else { "autre" };
                let unreadable = if *total_seconds > 1 { "illisibles" } else { "illisible" };
                format!("+ {count} {others}, {} {unreadable} en tout", duration(*total_seconds))
            }
            (Language::English, HeaderLine::MoreUnreadable { count, total_seconds }) => {
                format!("+ {count} more, {} unreadable in all", duration(*total_seconds))
            }
        }
    }

    fn band(&self, band: Band) -> String {
        match (self.0, band) {
            (Language::French, Band::NotAnalysed) => "hors analyse",
            (Language::English, Band::NotAnalysed) => "not analysed",
            (Language::French, Band::Unreadable) => "illisible",
            (Language::English, Band::Unreadable) => "unreadable",
        }
        .to_owned()
    }
}
