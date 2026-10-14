use std::error::Error;
use std::path::PathBuf;

use batdetect::jobs::Job;
use batdetect::writable::Unwritable;
use engine::settings;
use files::previous::Kept;
use render::config;

use crate::texts::Message;

#[cfg(test)]
mod tests;

const CAUSE_SEPARATOR: &str = ": ";

pub(crate) fn detection_refused(error: &settings::Error) -> Message {
    match *error {
        settings::Error::Setting { setting, value, bound } => Message::SettingRefused {
            option: format!("--{}", setting.flag()),
            bound,
            value,
        },
        settings::Error::Region { index, region } => Message::RegionRefused {
            index: index + 1,
            region: region.to_string(),
        },
        settings::Error::Areas { .. } => settings_refused(error),
    }
}

pub(crate) fn render_refused(error: &config::Error) -> Message {
    match *error {
        config::Error::Seconds {
            name: "clip_margin_s",
            value,
        } => Message::ClipMarginRefused { value },
        _ => settings_refused(error),
    }
}

pub(crate) fn settings_refused(error: &dyn Error) -> Message {
    Message::SettingsRefused { causes: causes(error) }
}

pub(crate) fn videos_refused(error: &dyn Error) -> Message {
    Message::VideosRefused { causes: causes(error) }
}

pub(crate) fn folders_refused(refused: &[Unwritable]) -> Message {
    let folders: Vec<String> = refused
        .iter()
        .map(|refusal| format!("{} ({})", refusal.folder.display(), refusal.source))
        .collect();
    Message::FoldersRefused {
        folders: folders.join(", "),
    }
}

pub(crate) fn version() -> &'static str {
    env!("BATDETECT_VERSION")
}

pub(crate) fn long_version(ffmpeg_version: &str, ffmpeg_license: &str) -> String {
    format!("{}\nFFmpeg {ffmpeg_version} — {ffmpeg_license}", version())
}

pub(crate) fn processed(job: &Job, tracks: usize) -> Message {
    Message::Processed {
        video: job.video.file_name().unwrap_or_default().to_string_lossy().into_owned(),
        tracks,
        dest: job.dest.display().to_string(),
    }
}

pub(crate) fn too_many_tracks(job: &Job, count: usize, limit: usize, annotated: bool) -> Message {
    Message::TooManyTracks {
        video: job.video.display().to_string(),
        count,
        limit,
        annotated,
    }
}

pub(crate) fn table_kept(job: &Job, kept: &Kept) -> Message {
    Message::TableKept {
        video: job.video.display().to_string(),
        table: kept.path.file_name().unwrap_or_default().to_string_lossy().into_owned(),
        written: !kept.ours,
    }
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

pub(crate) fn failed(job: &Job, error: &dyn Error) -> Message {
    Message::Failed {
        video: job.video.display().to_string(),
        causes: causes(error),
    }
}

pub(crate) fn internal_error(job: &Job, message: &str) -> Message {
    Message::InternalError {
        video: job.video.display().to_string(),
        message: message.to_owned(),
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
