use std::error::Error;
use std::path::PathBuf;

use batdetect::jobs::Job;
use engine::settings::{self, Setting};
use files::names::Part;
use render::config;
use summary::header::{Band, HeaderLine, Labels, duration};
use types::language::Language;

#[cfg(test)]
mod tests;

pub(crate) const ABOUT: &str = "Detects and tracks bats in thermal binocular videos";
pub(crate) const INPUTS_HELP: &str = "videos, or folders whose videos are all processed";
pub(crate) const OUT_DIR_HELP: &str = "folder that receives one folder of results per video";
pub(crate) const ZOOM_HELP: &str = "slowed, magnified clip: auto for small or faint tracks only, all, or none";
pub(crate) const ANNOTATED_HELP: &str = "also write the whole video with every track drawn";
pub(crate) const MAX_TRACKS_HELP: &str =
    "above this many tracks, the video counts as failed and gets no clips or annotated video; 0 disables";
pub(crate) const CLIP_MARGIN_HELP: &str = "time kept before and after each track in its clip; seconds";
pub(crate) const OSD_REGION_HELP: &str = "hides a moving on-screen display from the detection, as fractions of the frame; repeat for several; none by default";
pub(crate) const HELP_HELP: &str = "print help: -h the essential options, --help all of them";
pub(crate) const VERSION_HELP: &str = "print the version";
pub(crate) const DEFAULT_OPENING: &str = "[default:\u{a0}";
pub(crate) const POSSIBLE_OPENING: &str = "[possible values: ";
pub(crate) const DETECTION_HEADING: &str = "Settings that change the detection";
pub(crate) const THRESHOLD_HELP: &str = "smallest difference from the background that counts as a target; lower finds fainter targets and more noise; grey levels";
pub(crate) const MIN_HITS_HELP: &str =
    "tracks with fewer detections are dropped; lower keeps shorter tracks and more short flickers";
pub(crate) const MAX_BLOBS_HELP: &str = "frames with at least this many spots more than six times the usual count are \
     ignored, as when the binoculars move; 0 disables";
pub(crate) const WORK_WIDTH_HELP: &str =
    "width the frames are reduced to for detection; wider sees smaller targets but takes longer; pixels";
pub(crate) const NO_VIDEO: &str = "no video found";
const CAUSE_SEPARATOR: &str = ": ";

pub(crate) fn detection_refused(error: &settings::Error) -> String {
    match *error {
        settings::Error::Setting { setting, value, bound } => {
            format!("{} must be {bound}, not {value}", detection_option(setting))
        }
        settings::Error::Region { index, region } => format!(
            "--osd-region {} does not satisfy 0 ≤ x0 < x1 ≤ 1 and 0 ≤ y0 < y1 ≤ 1: {region}",
            index + 1
        ),
        settings::Error::Areas { .. } => error.to_string(),
    }
}

fn detection_option(setting: Setting) -> String {
    match setting {
        Setting::Threshold => "--threshold".to_owned(),
        Setting::WorkWidth => "--work-width".to_owned(),
        other => other.to_string(),
    }
}

pub(crate) fn render_refused(error: &config::Error) -> String {
    match *error {
        config::Error::Seconds {
            name: "clip_margin_s",
            value,
        } => format!("--clip-margin must be a finite number of seconds, at least 0, not {value}"),
        _ => error.to_string(),
    }
}

pub(crate) fn version() -> &'static str {
    env!("BATDETECT_VERSION")
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

pub(crate) fn too_many_tracks(language: Language, job: &Job, count: usize, limit: usize, annotated: bool) -> String {
    let video = job.video.display();
    let skipped = match (language, annotated) {
        (Language::French, false) => "extraits non écrits",
        (Language::French, true) => "extraits et vidéo annotée non écrits",
        (Language::English, false) => "clips skipped",
        (Language::English, true) => "clips and annotated video skipped",
    };
    match language {
        Language::French => format!("{video} : {count} pistes, au-delà de {limit} : {skipped}"),
        Language::English => format!("{video}: {count} tracks, above {limit}: {skipped}"),
    }
}

pub(crate) fn redone(language: Language, job: &Job, clips: &[PathBuf], cause: &dyn Error) -> String {
    let video = job.video.display();
    let names: Vec<String> = clips
        .iter()
        .map(|clip| clip.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .collect();
    let names = names.join(", ");
    let cause = causes(cause);
    match language {
        Language::French => {
            let written = if clips.len() > 1 { "réécrits" } else { "réécrit" };
            format!("{video} : {names} {written} avec openh264 après une erreur de VideoToolbox : {cause}")
        }
        Language::English => {
            format!("{video}: {names} written again with openh264 after a VideoToolbox error: {cause}")
        }
    }
}

pub(crate) fn failed(job: &Job, error: &dyn Error) -> String {
    format!("{}{CAUSE_SEPARATOR}{}", job.video.display(), causes(error))
}

pub(crate) fn internal_error(job: &Job, message: &str) -> String {
    format!(
        "{}{CAUSE_SEPARATOR}internal error{CAUSE_SEPARATOR}{message}",
        job.video.display()
    )
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
