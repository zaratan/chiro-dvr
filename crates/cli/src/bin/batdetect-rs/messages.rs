use std::error::Error;

use batdetect::jobs::Job;
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
